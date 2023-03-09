use std::collections::HashSet;

use serde::Serialize;
use tree_sitter_facade::{
    Node, Parser, Point, Query, QueryCapture, QueryCursor, QueryPredicate, Tree,
};

use crate::{
    atom_collection::AtomCollection, error::FormatterError, language::Language, FormatterResult,
};

/// Supported visualisation formats
#[derive(Clone, Copy, Debug)]
pub enum Visualisation {
    GraphViz,
    Json,
}

// 1-based text position, derived from tree_sitter::Point, for the sake of serialisation.
#[derive(Serialize)]
struct Position {
    row: u32,
    column: u32,
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
        let children = node.children(&mut walker).map(SyntaxNode::from).collect();

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

pub fn apply_query(
    input_content: &str,
    query_content: &str,
    language: Language,
) -> FormatterResult<AtomCollection> {
    let (tree, grammar) = parse(input_content, language)?;
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let query = Query::new(&grammar, query_content)
        .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;
    let mut cursor = QueryCursor::new();

    let capture_names = query.capture_names();

    // Match queries
    let mut matches: Vec<LocalQueryMatch> = Vec::new();
    for query_match in query.matches(&root, source, &mut cursor) {
        // This may provoke the collect bug described here:
        // https://github.com/tree-sitter/tree-sitter/issues/608
        let local_captures: Vec<QueryCapture> = query_match.captures().collect();

        matches.push(LocalQueryMatch {
            pattern_index: query_match.pattern_index(),
            captures: local_captures,
        })
    }

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    let specified_leaf_nodes: HashSet<usize> = collect_leaf_ids(&matches, &capture_names);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms = AtomCollection::collect_leafs(root, source, specified_leaf_nodes)?;

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

        let mut delimiter: Option<String> = None;
        let mut scope_id: Option<String> = None;

        for p in query.general_predicates(m.pattern_index) {
            if let Some(d) = handle_delimiter_predicate(&p)? {
                delimiter = Some(d);
            }
            if let Some(d) = handle_scope_id_predicate(&p)? {
                scope_id = Some(d);
            }
        }

        // If any capture is a do_nothing, then do nothing.
        if m.captures
            .iter()
            .map(|c| c.utf8_name(&capture_names))
            .any(|name| name == "do_nothing")
        {
            continue;
        }

        for c in m.captures {
            let name = c.utf8_name(&capture_names);
            atoms.resolve_capture(&name, &c.node(), delimiter.as_deref(), scope_id.as_deref())?;
        }
    }

    // Now apply all atoms in prepend and append to the leaf nodes.
    atoms.apply_prepends_and_appends();

    Ok(atoms)
}

// fn capture_name<'a>(query: &'a Query, capture: &QueryCapture) -> &'a str {
//     query.capture_names()[capture.index as usize].as_str()
// }

// A single "language" can correspond to multiple grammars.
// For instance, we have separate grammars for interfaces and implementation in OCaml.
// When the proper grammar cannot be inferred from the extension of the input file,
// this function tries to parse the data with every possible grammar.
// It returns the syntax tree of the first grammar that succeeds, along with said grammar,
// or the last error if all grammars fail.
pub fn parse(
    content: &str,
    language: Language,
) -> FormatterResult<(Tree, tree_sitter_facade::Language)> {
    let mut parser = Parser::new()?;

    language
        .grammars()
        .into_iter()
        .map(|grammar| {
            parser.set_language(&grammar).map_err(|_| {
                FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
            })?;

            let tree = parser
                .parse(content, None)?
                .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;

            // Fail parsing if we don't get a complete syntax tree.
            check_for_error_nodes(tree.root_node())?;

            Ok((tree, grammar))
        })
        .fold(
            Err(FormatterError::Internal(
                "Could not find any grammar".into(),
                None,
            )),
            Result::or,
        )
}

fn check_for_error_nodes(node: Node) -> FormatterResult<()> {
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
        check_for_error_nodes(child)?;
    }

    Ok(())
}

fn collect_leaf_ids(matches: &Vec<LocalQueryMatch>, capture_names: &[String]) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for m in matches {
        for c in &m.captures {
            if c.utf8_name(capture_names) == "leaf" {
                ids.insert(c.node().id());
            }
        }
    }
    ids
}

// TODO: Deduplicate these.

fn handle_delimiter_predicate(predicate: &QueryPredicate) -> FormatterResult<Option<String>> {
    let operator = &*predicate.utf8_operator();

    if let "delimiter!" = operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;

        Ok(Some(arg))
    } else {
        Ok(None)
    }
}

fn handle_scope_id_predicate(predicate: &QueryPredicate) -> FormatterResult<Option<String>> {
    let operator = &*predicate.utf8_operator();

    if let "scope_id!" = operator {
        let arg =
            predicate.args().into_iter().next().ok_or_else(|| {
                FormatterError::Query(format!("{operator} needs an argument"), None)
            })?;

        Ok(Some(arg))
    } else {
        Ok(None)
    }
}
