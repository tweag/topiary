use clap::ArgEnum;
use itertools::Itertools;
use pretty::RcDoc;
use std::cmp;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use test_log::test;
use tree_sitter::Point;
use tree_sitter::Tree;
use tree_sitter::{Node, Parser, Query, QueryCursor};

#[derive(ArgEnum, Clone, Debug)]
pub enum Language {
    Json,
    Ocaml,
    Rust,
}

pub fn formatter(
    input: &mut dyn io::Read,
    output: &mut dyn io::Write,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    // Read input and query
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    let query_str = &fs::read_to_string(Path::new(&str::to_lowercase(
        format!("languages/queries/{:?}.scm", language).as_str(),
    )))?;

    let grammar = grammar(language);
    let tree = parse(&content, grammar);
    let root = tree.root_node();
    let source = content.as_bytes();
    let query = Query::new(grammar, query_str).expect("Error parsing query file");

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

    // Detect first node on each line
    let line_start_columns = detect_line_start_columns(root);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms: Vec<Atom> = Vec::new();
    collect_leafs(root, &mut atoms, source, &specified_leaf_nodes, 0);

    log::debug!("List of atoms before formatting: {atoms:?}");

    // Formatting. If there are more than one capture per match, it generally
    // means that we want to use the last capture. For example
    // ((enum_item) @append_hardline . (line_comment)? @append_hardline)
    // means we want to append a hardline at the end, but we don't know if we get
    // a line_comment capture or not.

    for m in matches {
        if let Some(c) = m.captures.last() {
            let name = query.capture_names()[c.index as usize].clone();

            resolve_capture(
                name,
                &mut atoms,
                c.node,
                &multi_line_nodes,
                &blank_lines_before,
                &line_start_columns,
            );
        }
    }

    put_indent_ends_before_hardlines(&mut atoms);
    let atoms = clean_up_consecutive_spaces(&atoms);
    let atoms = trim_spaces_after_hardlines(&atoms);

    log::debug!("Final list of atoms: {atoms:?}");

    // Convert our list of atoms to a Doc
    let doc = atoms_to_doc(&mut 0, &atoms);
    let mut rendered = String::new();
    doc.render_fmt(usize::max_value(), &mut rendered)?;

    // Remove trailing spaces from lines
    let trimmed: String =
        Itertools::intersperse(rendered.split('\n').map(|line| line.trim_end()), "\n").collect();

    write!(output, "{trimmed}")?;

    Ok(())
}

/// A Node from tree-sitter is turned into into a list of atoms
#[derive(Clone, Debug, PartialEq)]
enum Atom {
    Hardline,
    IndentStart,
    IndentEnd,
    Leaf { content: String, id: usize },
    Literal(String),
    Softline { spaced: bool },
    Space,
}

fn grammar(language: Language) -> tree_sitter::Language {
    match language {
        Language::Json => tree_sitter_json::language(),
        Language::Ocaml => tree_sitter_ocaml::language_ocaml(),
        Language::Rust => tree_sitter_rust::language(),
    }
}

fn parse(content: &str, grammar: tree_sitter::Language) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(grammar).expect("Error loading grammar");
    let parsed = parser.parse(&content, None).expect("Could not parse input");
    parsed
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
) {
    let id = node.id();

    log::debug!(
        "CST node: {}{:?} - Named: {}",
        "  ".repeat(level),
        node,
        node.is_named()
    );

    if node.child_count() == 0 || specified_leaf_nodes.contains(&node.id()) {
        atoms.push(Atom::Leaf {
            content: String::from(node.utf8_text(source).expect("Source file not valid utf8")),
            id,
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source, &specified_leaf_nodes, level + 1);
        }
    }
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

fn atoms_to_doc<'a>(i: &mut usize, atoms: &'a Vec<Atom>) -> RcDoc<'a, ()> {
    let mut doc = RcDoc::nil();
    while *i < atoms.len() {
        let atom = &atoms[*i];
        if let Atom::IndentEnd = atom {
            return doc;
        } else {
            doc = doc.append(match atom {
                Atom::Leaf { content, .. } => RcDoc::text(content),
                Atom::Literal(s) => RcDoc::text(s),
                Atom::Hardline => RcDoc::hardline(),
                Atom::IndentEnd => unreachable!(),
                Atom::IndentStart => {
                    *i = *i + 1;
                    atoms_to_doc(i, atoms).nest(2)
                }
                Atom::Softline { .. } => unreachable!(),
                Atom::Space => RcDoc::space(),
            });
        }
        *i = *i + 1;
    }
    return doc;
}

fn resolve_capture(
    name: String,
    atoms: &mut Vec<Atom>,
    node: Node,
    multi_line_nodes: &HashSet<usize>,
    blank_lines_before: &HashSet<usize>,
    line_start_columns: &HashSet<Point>,
) {
    match name.as_ref() {
        "allow_blank_line_before" => {
            if blank_lines_before.contains(&node.id()) {
                atoms_prepend(Atom::Hardline, node, atoms, multi_line_nodes);
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
        let parent = node.parent();
        let parent_id = parent.expect("Parent node not found").id();

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
    detect_blank_lines_before_inner(node, &mut None)
}

fn detect_blank_lines_before_inner<'a>(
    node: Node<'a>,
    previous_node: &mut Option<Node<'a>>,
) -> HashSet<usize> {
    let mut ids = HashSet::new();

    if let Some(previous_node) = previous_node {
        // If two consequent nodes don't start immediately after each other,
        // there are blank lines between them, because no leaf nodes are
        // themselves multi-line.
        let previous_start = previous_node.start_position().row;
        let current_start = node.start_position().row;

        if current_start - previous_start > 1 {
            let id = node.id();
            ids.insert(id);
            log::debug!(
                "There are blank lines between {:?} and {:?}",
                previous_node,
                node
            );
        }
    }

    *previous_node = Some(node);

    for child in node.children(&mut node.walk()) {
        ids.extend(detect_blank_lines_before_inner(child, previous_node));
    }

    ids
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

fn clean_up_consecutive_spaces(atoms: &Vec<Atom>) -> Vec<Atom> {
    let filtered = atoms
        .split(|a| *a == Atom::Space)
        .filter(|chain| chain.len() > 0);

    Itertools::intersperse(filtered, &[Atom::Space])
        .flatten()
        .map(|a| a.clone())
        .collect_vec()
}

fn trim_spaces_after_hardlines(atoms: &Vec<Atom>) -> Vec<Atom> {
    let trimmed = atoms.split(|a| *a == Atom::Hardline).map(|slice| {
        slice
            .into_iter()
            .skip_while(|a| **a == Atom::Space)
            .collect::<Vec<_>>()
    });

    Itertools::intersperse(trimmed, vec![&Atom::Hardline])
        .flatten()
        .map(|a| a.clone())
        .collect_vec()
}

fn put_indent_ends_before_hardlines(atoms: &mut Vec<Atom>) {
    for i in 1..atoms.len() - 1 {
        if atoms[i] == Atom::Hardline && atoms[i + 1] == Atom::IndentEnd {
            atoms[i] = Atom::IndentEnd;
            atoms[i + 1] = Atom::Hardline;
        }
    }
}

#[test]
fn detect_multi_line_nodes_single_line() {
    let input = "enum OneLine { Leaf { content: String, id: usize, size: usize, }, Hardline { content: String, id: usize, }, Space, }";
    let grammar = grammar(Language::Rust);
    let tree = parse(input, grammar);
    let root = tree.root_node();
    let multi_line_nodes = detect_multi_line_nodes(root);
    assert!(multi_line_nodes.is_empty());
}

#[test]
fn detect_multi_line_nodes_expand_one_level() {
    let input = "enum ExpandOneLevel { Leaf { content: String, id: usize, size: usize, },\nHardline { content: String, id: usize, }, Space, }";
    let grammar = grammar(Language::Rust);
    let tree = parse(input, grammar);
    let root = tree.root_node();
    let multi_line_nodes = detect_multi_line_nodes(root);
    assert_eq!(3, multi_line_nodes.len());
}

#[test]
fn detect_multi_line_nodes_expand_two_levels() {
    let input = "enum ExpandTwoLevels { Leaf { content: String,\nid: usize, size: usize, }, Hardline { content: String, id: usize, }, Space, }";
    let grammar = grammar(Language::Rust);
    let tree = parse(input, grammar);
    let root = tree.root_node();
    let multi_line_nodes = detect_multi_line_nodes(root);
    assert_eq!(5, multi_line_nodes.len());
}

#[test]
fn detect_blank_lines_before_none() {
    let input = "enum OneLine { \nLeaf { \ncontent: String, \nid: usize, \nsize: usize, }, \nHardline { \ncontent: String, \nid: usize, }, \nSpace, \n}";
    let grammar = grammar(Language::Rust);
    let tree = parse(input, grammar);
    let root = tree.root_node();
    let blank_lines_before_nodes = detect_blank_lines_before(root);
    assert!(blank_lines_before_nodes.is_empty());
}

#[test]
fn detect_blank_lines_before_some() {
    let input = "enum OneLine { \nLeaf { \ncontent: String, \n\n\n\n\nid: usize, \nsize: usize, }, \n\nHardline { \ncontent: String, \nid: usize, }, \nSpace, \n}";
    let grammar = grammar(Language::Rust);
    let tree = parse(input, grammar);
    let root = tree.root_node();
    let blank_lines_before_nodes = detect_blank_lines_before(root);
    assert_eq!(2, blank_lines_before_nodes.len());
}
