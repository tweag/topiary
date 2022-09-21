use crate::error::FormatterError;
use crate::syntax_info::SyntaxInfo;
use crate::{Atom, Language, Result};
use std::collections::BTreeSet;
use tree_sitter::{Node, Parser, Query, QueryCursor, QueryPredicate, QueryPredicateArg, Tree};

pub struct QueryResult {
    pub atoms: Vec<Atom>,
    pub indent_level: isize,
}

pub fn apply_query(
    input_content: &str,
    query_content: &str,
    language: Language,
) -> Result<QueryResult> {
    let grammar = grammar(language);
    let tree = parse(input_content, grammar)?;
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let query = Query::new(grammar, query_content)
        .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;
    let mut indent_level = 2;

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    // TODO: Doesn't need to be ordered, can be just a HashSet.
    let specified_leaf_nodes: BTreeSet<usize> = collect_leaf_ids(&query, root, source);

    // Match queries
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root, source);

    // Detect important aspects, such as line breaks, from input syntax.
    let syntax = SyntaxInfo::detect(root);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms: Vec<Atom> = Vec::new();
    collect_leafs(root, &mut atoms, source, &specified_leaf_nodes, 0)?;

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
        let mut delimiter: Option<String> = None;

        for p in query.general_predicates(m.pattern_index) {
            if let Some(d) = handle_delimiter_predicate(p)? {
                delimiter = Some(d);
            }
            if let Some(il) = handle_indent_level_predicate(p)? {
                indent_level = il;
            }
        }

        if let Some(c) = m.captures.last() {
            let name = query.capture_names()[c.index as usize].clone();
            syntax.resolve_capture(name, &mut atoms, c.node, delimiter.as_deref())?;
        }
    }

    Ok(QueryResult {
        atoms,
        indent_level,
    })
}

fn grammar(language: Language) -> tree_sitter::Language {
    match language {
        Language::Json => tree_sitter_json::language(),
        Language::Ocaml => tree_sitter_ocaml::language_ocaml(),
        Language::Rust => tree_sitter_rust::language(),
    }
}

fn parse(content: &str, grammar: tree_sitter::Language) -> Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(grammar).map_err(|_| {
        FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
    })?;

    parser
        .parse(&content, None)
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))
}

fn collect_leafs<'a>(
    node: Node,
    atoms: &mut Vec<Atom>,
    source: &'a [u8],
    specified_leaf_nodes: &BTreeSet<usize>,
    level: usize,
) -> Result<()> {
    let id = node.id();

    log::debug!(
        "CST node: {}{:?} - Named: {}",
        "  ".repeat(level),
        node,
        node.is_named()
    );

    if node.child_count() == 0 || specified_leaf_nodes.contains(&node.id()) {
        atoms.push(Atom::Leaf {
            content: String::from(node.utf8_text(source)?),
            id,
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source, specified_leaf_nodes, level + 1)?;
        }
    }

    Ok(())
}

fn collect_leaf_ids<'a>(query: &Query, root: Node, source: &'a [u8]) -> BTreeSet<usize> {
    let mut ids = BTreeSet::new();

    // TODO: Should probably use the same cursor as above
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, root, source);

    for m in matches {
        for c in m.captures {
            if query.capture_names()[c.index as usize] == "leaf" {
                ids.insert(c.node.id());
            }
        }
    }
    ids
}

fn handle_indent_level_predicate(predicate: &QueryPredicate) -> Result<Option<isize>> {
    let operator = &*predicate.operator;

    if let "indent-level!" = operator {
        let arg = predicate
            .args
            .first()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument"), None))?;

        if let QueryPredicateArg::String(s) = arg {
            Ok(Some(s.parse().map_err(|_| {
                FormatterError::Query(
                    format!("{operator} needs a numeric argument, but got '{s}'."),
                    None,
                )
            })?))
        } else {
            Err(FormatterError::Query(
                format!("{operator} needs a numeric argument, but got {arg:?}."),
                None,
            ))
        }
    } else {
        Ok(None)
    }
}

fn handle_delimiter_predicate(predicate: &QueryPredicate) -> Result<Option<String>> {
    let operator = &*predicate.operator;

    if let "delimiter!" = operator {
        let arg = predicate
            .args
            .first()
            .ok_or_else(|| FormatterError::Query(format!("{operator} needs an argument"), None))?;

        if let QueryPredicateArg::String(s) = arg {
            Ok(Some(s.to_string()))
        } else {
            Err(FormatterError::Query(
                format!("{operator} needs a string argument, but got {arg:?}."),
                None,
            ))
        }
    } else {
        Ok(None)
    }
}
