//! Tree-sitter API related functions.
//! Notably, this module is not responsible for managing the actual grammars.
use crate::atom_collection::AtomCollection;
use crate::error::FormatterError;
use crate::language::Language;
use crate::FormatterResult;
use std::collections::BTreeSet;
use tree_sitter::{
    Node, Parser, Query, QueryCapture, QueryCursor, QueryPredicate, QueryPredicateArg, Tree,
};

pub fn apply_query(
    input_content: &str,
    query_content: &str,
    language: &Language,
) -> FormatterResult<AtomCollection> {
    let grammar = language.get_tree_sitter_language()?;
    let tree = parse(input_content, grammar)?;
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let query = Query::new(grammar, query_content)
        .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;

    // Fail formatting if we don't get a complete syntax tree.
    check_for_error_nodes(root)?;

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    // TODO: Doesn't need to be ordered, can be just a HashSet.
    let specified_leaf_nodes: BTreeSet<usize> = collect_leaf_ids(&query, root, source);

    // Match queries
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root, source);

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

        for p in query.general_predicates(m.pattern_index) {
            if let Some(d) = handle_delimiter_predicate(p)? {
                delimiter = Some(d);
            }
        }

        // If any capture is a do_nothing, then do nothing.
        if m.captures
            .iter()
            .map(|c| capture_name(&query, c))
            .any(|name| name == "do_nothing")
        {
            continue;
        }

        for c in m.captures {
            let name = capture_name(&query, c);
            atoms.resolve_capture(name, c.node, delimiter.as_deref())?;
        }
    }

    // Now apply all atoms in prepend and append to the leaf nodes.
    atoms.apply_prepends_and_appends();

    Ok(atoms)
}

fn capture_name<'a, 'b>(query: &'a Query, capture: &'b QueryCapture) -> &'a str {
    query.capture_names()[capture.index as usize].as_str()
}

fn parse(content: &str, grammar: tree_sitter::Language) -> FormatterResult<Tree> {
    let mut parser = Parser::new();
    parser.set_language(grammar).map_err(|_| {
        FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
    })?;

    parser
        .parse(&content, None)
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))
}

fn check_for_error_nodes(node: Node) -> FormatterResult<()> {
    if node.kind() == "ERROR" {
        let start = node.start_position();
        let end = node.end_position();

        // Report 1-based lines and columns.
        return Err(FormatterError::Parsing {
            start_line: start.row + 1,
            start_column: start.column + 1,
            end_line: end.row + 1,
            end_column: end.column + 1,
        });
    }

    for child in node.children(&mut node.walk()) {
        check_for_error_nodes(child)?;
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

fn handle_delimiter_predicate(predicate: &QueryPredicate) -> FormatterResult<Option<String>> {
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
