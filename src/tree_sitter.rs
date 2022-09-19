use crate::error::FormatterError;
use crate::error::ReadingError;
use crate::Atom;
use crate::Language;
use crate::Result;
use std::cmp;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Point;
use tree_sitter::Query;
use tree_sitter::QueryCursor;
use tree_sitter::QueryPredicate;
use tree_sitter::QueryPredicateArg;
use tree_sitter::Tree;

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
    let tree = parse(&input_content, grammar)?;
    let root = tree.root_node();
    let source = input_content.as_bytes();
    let query = Query::new(grammar, &query_content)
        .map_err(|e| FormatterError::Query("Error parsing query file".into(), Some(e)))?;
    let mut indent_level = 2;

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    // TODO: Doesn't need to be ordered, can be just a HashSet.
    let specified_leaf_nodes: BTreeSet<usize> = collect_leaf_ids(&query, root, source);

    // Match queries
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root, source);

    // Detect user specified line breaks
    let multi_line_nodes = detect_multi_line_nodes(root);
    let blank_lines_before = detect_blank_lines_before(root);
    let (line_break_before, line_break_after) = detect_line_break_before_and_after(root);

    // Detect first node on each line
    let line_start_columns = detect_line_start_columns(root);

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
        for p in query.general_predicates(m.pattern_index) {
            handle_predicate(p, &mut indent_level)?;
        }

        if let Some(c) = m.captures.last() {
            let name = query.capture_names()[c.index as usize].clone();

            resolve_capture(
                name,
                &mut atoms,
                c.node,
                &multi_line_nodes,
                &blank_lines_before,
                &line_break_before,
                &line_break_after,
                &line_start_columns,
            );
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

    parser.parse(&content, None).ok_or(FormatterError::Internal(
        "Could not parse input".into(),
        None,
    ))
}

/// Given a node, returns the id of the first leaf in the subtree.
fn first_leaf(node: Node) -> Node {
    if node.child_count() == 0 {
        node
    } else {
        first_leaf(node.child(0).unwrap())
    }
}

/// Given a node, returns the id of the last leaf in the subtree.
fn last_leaf(node: Node) -> Node {
    let nr_children = node.child_count();
    if nr_children == 0 {
        node
    } else {
        last_leaf(node.child(nr_children - 1).unwrap())
    }
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
            content: String::from(
                node.utf8_text(source)
                    .map_err(|e| FormatterError::Reading(ReadingError::Utf8(e)))?,
            ),
            id,
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source, &specified_leaf_nodes, level + 1)?;
        }
    }

    Ok(())
}

/// Finds the matching node in the atoms and returns the index
/// TODO: Error
fn find_node(node: Node, atoms: &mut Vec<Atom>) -> usize {
    let mut target_node = node;
    loop {
        for (i, node) in atoms.iter().enumerate() {
            match node {
                Atom::Leaf { id, .. } => {
                    if *id == target_node.id() {
                        return i;
                    }
                }
                _ => continue,
            }
        }
        target_node = match node.parent() {
            Some(p) => p,
            None => unreachable!(),
        }
    }
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

fn handle_predicate(predicate: &QueryPredicate, indent_level: &mut isize) -> Result<()> {
    let operator = &*predicate.operator;

    match operator {
        "indent-level!" => {
            let arg = predicate.args.first().ok_or(FormatterError::Query(
                "indent-level! needs an argument".into(),
                None,
            ))?;

            match arg {
                QueryPredicateArg::String(s) => {
                    *indent_level = s.parse().map_err(|_| {
                        FormatterError::Query(
                            format!("indent-level! needs a numeric argument, but got '{s}'."),
                            None,
                        )
                    })?;
                    Ok(())
                }
                _ => Err(FormatterError::Query(
                    format!("indent-level! needs a numeric argument, but got {arg:?}."),
                    None,
                )),
            }
        }
        _ => Err(FormatterError::Query(
            format!("Unexpected predicate '{operator}'"),
            None,
        )),
    }
}

fn resolve_capture(
    name: String,
    atoms: &mut Vec<Atom>,
    node: Node,
    multi_line_nodes: &HashSet<usize>,
    blank_lines_before: &HashSet<usize>,
    line_break_before: &HashSet<usize>,
    line_break_after: &HashSet<usize>,
    line_start_columns: &HashSet<Point>,
) {
    match name.as_ref() {
        "allow_blank_line_before" => {
            if blank_lines_before.contains(&node.id()) {
                atoms_prepend(Atom::Blankline, node, atoms, multi_line_nodes);
            }
        }
        "append_comma" => atoms_append(
            Atom::Literal(",".to_string()),
            node,
            atoms,
            multi_line_nodes,
        ),
        "append_empty_softline" => atoms_append(
            Atom::Softline { spaced: false },
            node,
            atoms,
            multi_line_nodes,
        ),
        "append_hardline" => atoms_append(Atom::Hardline, node, atoms, multi_line_nodes),
        "append_indent_start" => atoms_append(Atom::IndentStart, node, atoms, multi_line_nodes),
        "append_indent_end" => atoms_append(Atom::IndentEnd, node, atoms, multi_line_nodes),
        "append_input_softline" => {
            let space = if line_break_after.contains(&node.id()) {
                Atom::Hardline
            } else {
                Atom::Space
            };

            atoms_append(space, node, atoms, multi_line_nodes);
        }
        "append_space" => atoms_append(Atom::Space, node, atoms, multi_line_nodes),
        "append_spaced_softline" => atoms_append(
            Atom::Softline { spaced: true },
            node,
            atoms,
            multi_line_nodes,
        ),
        "prepend_empty_softline" => atoms_prepend(
            Atom::Softline { spaced: false },
            node,
            atoms,
            multi_line_nodes,
        ),
        "prepend_indent_start" => atoms_prepend(Atom::IndentStart, node, atoms, multi_line_nodes),
        "prepend_indent_end" => atoms_prepend(Atom::IndentEnd, node, atoms, multi_line_nodes),
        "prepend_input_softline" => {
            let space = if line_break_before.contains(&node.id()) {
                Atom::Hardline
            } else {
                Atom::Space
            };

            atoms_prepend(space, node, atoms, multi_line_nodes);
        }
        "prepend_space" => atoms_prepend(Atom::Space, node, atoms, multi_line_nodes),
        "prepend_space_unless_first_on_line" => {
            if !line_start_columns.contains(&node.start_position()) {
                atoms_prepend(Atom::Space, node, atoms, multi_line_nodes)
            }
        }
        "prepend_spaced_softline" => atoms_prepend(
            Atom::Softline { spaced: true },
            node,
            atoms,
            multi_line_nodes,
        ),
        // Skip over leafs
        _ => return,
    }
}

fn atoms_append(atom: Atom, node: Node, atoms: &mut Vec<Atom>, multi_line_nodes: &HashSet<usize>) {
    if let Some(atom) = expand_softline(atom, node, multi_line_nodes) {
        let target_node = last_leaf(node);
        let index = find_node(target_node, atoms);
        if index > atoms.len() {
            atoms.push(atom);
        } else {
            atoms.insert(index + 1, atom);
        }
    }
}

fn atoms_prepend(atom: Atom, node: Node, atoms: &mut Vec<Atom>, multi_line_nodes: &HashSet<usize>) {
    if let Some(atom) = expand_softline(atom, node, multi_line_nodes) {
        let target_node = first_leaf(node);
        let index = find_node(target_node, atoms);
        atoms.insert(index, atom);
    }
}

fn expand_softline(atom: Atom, node: Node, multi_line_nodes: &HashSet<usize>) -> Option<Atom> {
    if let Atom::Softline { spaced } = atom {
        if let Some(parent) = node.parent() {
            let parent_id = parent.id();

            if multi_line_nodes.contains(&parent_id) {
                log::debug!(
                    "Expanding softline to hardline in node {:?} with parent {}: {:?}",
                    node,
                    parent_id,
                    parent
                );
                Some(Atom::Hardline)
            } else if spaced {
                log::debug!(
                    "Expanding softline to space in node {:?} with parent {}: {:?}",
                    node,
                    parent_id,
                    parent
                );
                Some(Atom::Space)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some(atom)
    }
}

fn detect_multi_line_nodes(node: Node) -> HashSet<usize> {
    let mut ids = HashSet::new();

    for child in node.children(&mut node.walk()) {
        ids.extend(detect_multi_line_nodes(child));
    }

    let start_line = node.start_position().row;
    let end_line = node.end_position().row;

    if end_line > start_line {
        let id = node.id();
        ids.insert(id);
        log::debug!("Multi-line node {}: {:?}", id, node,);
    }

    ids
}

fn detect_blank_lines_before(node: Node) -> HashSet<usize> {
    detect_line_breaks_inner(node, 2, &mut None).0
}

fn detect_line_break_before_and_after(node: Node) -> (HashSet<usize>, HashSet<usize>) {
    detect_line_breaks_inner(node, 1, &mut None)
}

fn detect_line_breaks_inner<'a>(
    node: Node<'a>,
    minimum_line_breaks: usize,
    previous_node: &mut Option<Node<'a>>,
) -> (HashSet<usize>, HashSet<usize>) {
    let mut nodes_with_breaks_before = HashSet::new();
    let mut nodes_with_breaks_after = HashSet::new();

    if let Some(previous_node) = previous_node {
        let previous_end = previous_node.end_position().row;
        let current_start = node.start_position().row;

        if current_start >= previous_end + minimum_line_breaks {
            nodes_with_breaks_before.insert(node.id());
            nodes_with_breaks_after.insert(previous_node.id());

            log::debug!(
                "There are at least {} blank lines between {:?} and {:?}",
                minimum_line_breaks,
                previous_node,
                node
            );
        }
    }

    *previous_node = Some(node);

    for child in node.children(&mut node.walk()) {
        let (before, after) = detect_line_breaks_inner(child, minimum_line_breaks, previous_node);
        nodes_with_breaks_before.extend(before);
        nodes_with_breaks_after.extend(after);
    }

    (nodes_with_breaks_before, nodes_with_breaks_after)
}

fn detect_line_start_columns(node: Node) -> HashSet<Point> {
    let mut line_start_columns = HashMap::new();

    detect_line_start_columns_inner(node, &mut line_start_columns);

    line_start_columns
        .into_iter()
        .map(|kv| Point::new(kv.0, kv.1))
        .collect()
}

fn detect_line_start_columns_inner(node: Node, line_start_columns: &mut HashMap<usize, usize>) {
    let position = node.start_position();
    let row = position.row;
    let column = position.column;

    let stored_column = line_start_columns.entry(row).or_insert(usize::max_value());
    *stored_column = cmp::min(*stored_column, column);

    for child in node.children(&mut node.walk()) {
        detect_line_start_columns_inner(child, line_start_columns);
    }
}
