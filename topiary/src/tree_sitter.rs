use std::collections::HashSet;

use serde::Serialize;
use tree_sitter_facade::{
    InputEdit, Language, Node, Parser, Point, Query, QueryCapture, QueryCursor, QueryPredicate,
    Tree, TreeCursor,
};

use crate::{
    atom_collection::{AtomCollection, CommentStream, QueryPredicates},
    error::FormatterError,
    FormatterResult,
};

/// Supported visualisation formats
#[derive(Clone, Copy, Debug)]
pub enum Visualisation {
    GraphViz,
    Json,
}

/// Refers to a position within the code. Used for error reporting, and for
/// comparing input with formatted output. The numbers are 1-based, because that
/// is how editors usually refer to a position. Derived from tree_sitter::Point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub row: u32,
    pub column: u32,
}

/// Topiary often needs both the tree-sitter `Query` and the original content
/// beloging to the file from which the query was parsed. This struct is a simple
/// convenience wrapper that combines the `Query` with its original string.
pub struct TopiaryQuery {
    pub query: Query,
    pub query_content: String,
}

impl TopiaryQuery {
    /// Creates a new `TopiaryQuery` from a tree-sitter language/grammar and the
    /// contents of the query file.
    ///
    /// # Errors
    ///
    /// This function will return an error if tree-sitter failed to parse the
    /// query file.
    pub fn new(
        grammar: &tree_sitter_facade::Language,
        query_content: &str,
    ) -> FormatterResult<TopiaryQuery> {
        let query = Query::new(grammar, query_content)
            .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;

        Ok(TopiaryQuery {
            query,
            query_content: query_content.to_owned(),
        })
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            row: point.row() + 1,
            column: point.column() + 1,
        }
    }
}

// Simplified syntactic node struct, for the sake of serialisation.
#[derive(Serialize)]
pub struct SyntaxNode {
    #[serde(skip_serializing)]
    pub id: usize,

    pub kind: String,
    pub is_named: bool,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    start: Position,
    end: Position,

    pub children: Vec<SyntaxNode>,
}

impl From<Node<'_>> for SyntaxNode {
    fn from(node: Node) -> Self {
        let mut walker = node.walk();
        let children = node.children(&mut walker).map(Self::from).collect();

        Self {
            id: node.id(),

            kind: node.kind().into(),
            is_named: node.is_named(),
            is_extra: node.is_extra(),
            is_error: node.is_error(),
            is_missing: node.is_missing(),
            start: node.start_position().into(),
            end: node.end_position().into(),

            children,
        }
    }
}

#[derive(Debug)]
// A struct to statically store the public fields of query match results,
// to avoid running queries twice.
struct LocalQueryMatch<'a> {
    pattern_index: u32,
    captures: Vec<QueryCapture<'a>>,
}

fn is_comment(node: &Node) -> bool {
    node.is_extra() && node.kind().to_string().contains("comment")
}

fn find_comments<'a>(node: Node<'a>, comments: &mut Vec<Node<'a>>) -> () {
    if is_comment(&node) {
        comments.push(node);
    } else {
        let mut walker = node.walk();
        for child in node.children(&mut walker) {
            find_comments(child, comments)
        }
    }
}

enum Anchor<T> {
    AnchorBefore(T),
    AnchorAfter(T),
}

fn next_non_comment<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    let mut temp_node: Node<'tree> = node;
    loop {
        match temp_node.next_sibling() {
            Some(sibling) => {
                if !is_comment(&sibling) {
                    return Some(sibling);
                }
                temp_node = sibling;
            }
            None => return None,
        }
    }
}

fn previous_non_comment<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    let mut temp_node = node;
    loop {
        match temp_node.prev_sibling() {
            Some(sibling) => {
                if !is_comment(&sibling) {
                    return Some(sibling);
                }
                temp_node = sibling
            }
            None => return None,
        }
    }
}

// Use the following heuristics to find a comment's anchor:
// If the comment is only prefixed by blank symbols on its line, then the anchor is the
// next non-comment sibling node.
// Otherwise, the anchor is the previous non-comment sibling node.
// If there is no such node, we anchor to the first non-comment sibling node
// in the other direction.
fn find_anchor<'tree>(
    node: &'tree Node<'tree>,
    input: &str,
) -> FormatterResult<Anchor<Node<'tree>>> {
    let point = node.start_position();
    let mut lines = input.lines();
    let prefix = lines
        .nth(point.row() as usize)
        .map(|line| &line[..point.column() as usize])
        .ok_or_else(|| {
            FormatterError::Internal(
                format!(
                    "Trying to access nonexistent line {} in text:\n{}",
                    point.row(),
                    input,
                ),
                None,
            )
        })?;
    if prefix.trim_start() == "" {
        if let Some(anchor) = next_non_comment(node.clone()) {
            return Ok(Anchor::AnchorAfter(anchor));
        } else if let Some(anchor) = previous_non_comment(node.clone()) {
            return Ok(Anchor::AnchorBefore(anchor));
        } else {
            return Err(FormatterError::Internal(
                format!("Could find no anchor for comment {node:?}",),
                None,
            ));
        }
    } else {
        if let Some(anchor) = previous_non_comment(node.clone()) {
            return Ok(Anchor::AnchorBefore(anchor));
        } else if let Some(anchor) = next_non_comment(node.clone()) {
            return Ok(Anchor::AnchorAfter(anchor));
        } else {
            return Err(FormatterError::Internal(
                format!("Could find no anchor for comment {node:?}",),
                None,
            ));
        }
    }
}

// TODO: store comments instead of discarding them
fn extract_comments<'a>(
    tree: Tree,
    input: &str,
    grammar: &Language,
) -> FormatterResult<(Tree, String)> {
    // let mut comment_stream: CommentStream = CommentStream::new();
    let mut comments: Vec<Node> = Vec::new();
    let mut new_input: String = input.to_string();
    let mut new_tree: Tree = tree;
    find_comments(new_tree.root_node(), &mut comments);
    comments.sort_by_key(|node| node.start_byte());
    comments.reverse();
    let mut edits: Vec<InputEdit> = Vec::new();
    for node in comments {
        match find_anchor(&node, input)? {
            Anchor::AnchorBefore(anchor) => {
                log::debug!("Anchor precedes comment {:?}:\n{anchor:?}", &node)
            }
            Anchor::AnchorAfter(anchor) => {
                log::debug!("Anchor follows comment {:?}:\n{anchor:?}", &node)
            }
        }
        new_input.replace_range((node.start_byte() as usize)..(node.end_byte() as usize), "");
        let edit = InputEdit::new(
            node.start_byte(),
            node.end_byte(),
            node.start_byte(),
            &node.start_position(),
            &node.end_position(),
            &node.start_position(),
        );
        edits.push(edit);
    }
    for edit in edits {
        new_tree.edit(&edit);
    }
    new_tree = reparse(new_tree, new_input.as_str(), grammar)?;
    Ok((new_tree, new_input))
}

/// Applies a query to an input content and returns a collection of atoms.
///
/// # Errors
///
/// This function can return an error if:
/// - The input content cannot be parsed by the grammar.
/// - The query content cannot be parsed by the grammar.
/// - The input exhaustivity check fails.
/// - A found predicate could not be parsed or is malformed.
/// - A unknown capture name was encountered in the query.
pub fn apply_query(
    input_content: &str,
    query: &TopiaryQuery,
    grammar: &tree_sitter_facade::Language,
    tolerate_parsing_errors: bool,
    should_check_input_exhaustivity: bool,
) -> FormatterResult<AtomCollection> {
    let (tree, grammar) = parse(input_content, grammar, tolerate_parsing_errors)?;

    // Remove comments in a separate stream before applying queries
    let (tree, new_input) = extract_comments(tree, input_content, grammar)?;
    let source = new_input.as_bytes();
    let root = tree.root_node();
    // log::debug!("{tree:?}");
    // return Err(FormatterError::Internal("TOTAL FAILURE".into(), None));

    // Match queries
    let mut cursor = QueryCursor::new();
    let mut matches: Vec<LocalQueryMatch> = Vec::new();
    let capture_names = query.query.capture_names();

    for query_match in query.query.matches(&root, source, &mut cursor) {
        let local_captures: Vec<QueryCapture> = query_match.captures().collect();

        matches.push(LocalQueryMatch {
            pattern_index: query_match.pattern_index(),
            captures: local_captures,
        });
    }

    if should_check_input_exhaustivity {
        let ref_match_count = matches.len();
        check_input_exhaustivity(ref_match_count, query, grammar, &root, source)?;
    }

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    let specified_leaf_nodes: HashSet<usize> = collect_leaf_ids(&matches, &capture_names);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms = AtomCollection::collect_leafs(&root, source, specified_leaf_nodes)?;

    log::debug!("List of atoms before formatting: {atoms:?}");

    // If there are more than one capture per match, it generally means that we
    // want to use the last capture. For example
    // (
    //   (enum_item) @append_hardline .
    //   (line_comment)? @append_hardline
    // )
    // means we want to append a hardline at
    // the end, but we don't know if we get a line_comment capture or not.

    for m in matches {
        log::debug!("Processing match: {m:?}");

        let mut predicates = QueryPredicates::default();

        for p in query.query.general_predicates(m.pattern_index) {
            predicates = handle_predicate(&p, &predicates)?;
        }
        check_predicates(&predicates)?;

        // If any capture is a do_nothing, then do nothing.
        if m.captures
            .iter()
            .map(|c| c.name(&capture_names))
            .any(|name| name == "do_nothing")
        {
            continue;
        }

        for c in m.captures {
            let name = c.name(&capture_names);
            atoms.resolve_capture(&name, &c.node(), &predicates)?;
        }
    }

    // Now apply all atoms in prepend and append to the leaf nodes.
    atoms.apply_prepends_and_appends();

    Ok(atoms)
}

// A single "language" can correspond to multiple grammars.
// For instance, we have separate grammars for interfaces and implementation in OCaml.
// When the proper grammar cannot be inferred from the extension of the input file,
// this function tries to parse the data with every possible grammar.
// It returns the syntax tree of the first grammar that succeeds, along with said grammar,
// or the last error if all grammars fail.
pub fn parse<'a>(
    content: &str,
    grammar: &'a tree_sitter_facade::Language,
    tolerate_parsing_errors: bool,
) -> FormatterResult<(Tree, &'a tree_sitter_facade::Language)> {
    let mut parser = Parser::new()?;
    parser.set_language(grammar).map_err(|_| {
        FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
    })?;

    let tree = parser
        .parse(content, None)?
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;

    // Fail parsing if we don't get a complete syntax tree.
    if !tolerate_parsing_errors {
        check_for_error_nodes(&tree.root_node())?;
    }

    Ok((tree, grammar))
}

fn reparse(
    old_tree: Tree,
    content: &str,
    grammar: &tree_sitter_facade::Language,
) -> FormatterResult<Tree> {
    let mut parser = Parser::new()?;
    parser.set_language(grammar)?;
    let tree = parser
        .parse(content, Some(&old_tree))?
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;
    Ok(tree)
}

fn check_for_error_nodes(node: &Node) -> FormatterResult<()> {
    if node.kind() == "ERROR" {
        let start = node.start_position();
        let end = node.end_position();

        // Report 1-based lines and columns.
        return Err(FormatterError::Parsing {
            start_line: start.row() + 1,
            start_column: start.column() + 1,
            end_line: end.row() + 1,
            end_column: end.column() + 1,
        });
    }

    for child in node.children(&mut node.walk()) {
        check_for_error_nodes(&child)?;
    }

    Ok(())
}

/// Collects the IDs of all leaf nodes in a set of query matches.
///
/// This function takes a slice of `LocalQueryMatch` and a slice of capture names,
/// and returns a `HashSet` of node IDs that are matched by the "leaf" capture name.
fn collect_leaf_ids(matches: &[LocalQueryMatch], capture_names: &[String]) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for m in matches {
        for c in &m.captures {
            if c.name(capture_names) == "leaf" {
                ids.insert(c.node().id());
            }
        }
    }
    ids
}

/// Handles a query predicate and returns a new set of query predicates with the corresponding field updated.
///
/// # Arguments
///
/// * `predicate` - A reference to a `QueryPredicate` object that represents a predicate in a query pattern.
/// * `predicates` - A reference to a `QueryPredicates` object that holds the current state of the query predicates.
///
/// # Returns
///
/// A `FormatterResult` that contains either a new `QueryPredicates` object with the updated field, or a `FormatterError` if the predicate is invalid or missing an argument.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The predicate operator is not one of the supported ones.
/// * The predicate operator requires an argument but none is provided.
fn handle_predicate(
    predicate: &QueryPredicate,
    predicates: &QueryPredicates,
) -> FormatterResult<QueryPredicates> {
    let operator = &*predicate.operator();
    if "delimiter!" == operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;
        Ok(QueryPredicates {
            delimiter: Some(arg),
            ..predicates.clone()
        })
    } else if "scope_id!" == operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;
        Ok(QueryPredicates {
            scope_id: Some(arg),
            ..predicates.clone()
        })
    } else if "single_line_only!" == operator {
        Ok(QueryPredicates {
            single_line_only: true,
            ..predicates.clone()
        })
    } else if "multi_line_only!" == operator {
        Ok(QueryPredicates {
            multi_line_only: true,
            ..predicates.clone()
        })
    } else if "single_line_scope_only!" == operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;
        Ok(QueryPredicates {
            single_line_scope_only: Some(arg),
            ..predicates.clone()
        })
    } else if "multi_line_scope_only!" == operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;
        Ok(QueryPredicates {
            multi_line_scope_only: Some(arg),
            ..predicates.clone()
        })
    } else {
        Ok(predicates.clone())
    }
}

/// Checks the validity of the query predicates.
///
/// This function ensures that the query predicates do not contain more than one
/// of the following: #single_line_only, #multi_line_only, #single_line_scope_only,
/// or #multi_line_scope_only. These predicates are incompatible with each other
/// and would result in an invalid query.
///
/// # Arguments
///
/// * `predicates` - A reference to a QueryPredicates struct that holds the query predicates.
///
/// # Errors
///
/// If the query predicates contain more than one incompatible predicate, this function
/// returns a FormatterError::Query with a descriptive message.
fn check_predicates(predicates: &QueryPredicates) -> FormatterResult<()> {
    let mut incompatible_predicates = 0;
    if predicates.single_line_only {
        incompatible_predicates += 1;
    }
    if predicates.multi_line_only {
        incompatible_predicates += 1;
    }
    if predicates.single_line_scope_only.is_some() {
        incompatible_predicates += 1;
    }
    if predicates.multi_line_scope_only.is_some() {
        incompatible_predicates += 1;
    }
    if incompatible_predicates > 1 {
        Err(FormatterError::Query(
            "A query can contain at most one #single/multi_line[_scope]_only! predicate".into(),
            None,
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Check if the input tests all patterns in the query, by successively disabling
/// all patterns. If disabling a pattern does not decrease the number of matches,
/// then that pattern originally matched nothing in the input.
fn check_input_exhaustivity(
    ref_match_count: usize,
    original_query: &TopiaryQuery,
    grammar: &tree_sitter_facade::Language,
    root: &Node,
    source: &[u8],
) -> FormatterResult<()> {
    let pattern_count = original_query.query.pattern_count();
    let query_content = &original_query.query_content;
    // This particular test avoids a SIGSEGV error that occurs when trying
    // to count the matches of an empty query (see #481)
    if pattern_count == 1 {
        if ref_match_count == 0 {
            return Err(FormatterError::PatternDoesNotMatch(query_content.into()));
        } else {
            return Ok(());
        }
    }
    for i in 0..pattern_count {
        // We don't need to use TopiaryQuery in this test since we have no need
        // for duplicate versions of the query_content string, instead we create the query
        // manually.
        let mut query = Query::new(grammar, query_content)
            .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;
        query.disable_pattern(i);
        let mut cursor = QueryCursor::new();
        let match_count = query.matches(root, source, &mut cursor).count();
        if match_count == ref_match_count {
            let index_start = query.start_byte_for_pattern(i);
            let index_end = if i == pattern_count - 1 {
                query_content.len()
            } else {
                query.start_byte_for_pattern(i + 1)
            };
            let pattern_content = &query_content[index_start..index_end];
            return Err(FormatterError::PatternDoesNotMatch(pattern_content.into()));
        }
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn check_input_exhaustivity(
    _ref_match_count: usize,
    _original_query: &TopiaryQuery,
    _grammar: &tree_sitter_facade::Language,
    _root: &Node,
    _source: &[u8],
) -> FormatterResult<()> {
    unimplemented!();
}
