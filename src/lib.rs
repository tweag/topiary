use pretty::RcDoc;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use tree_sitter::{Node, Parser, Query, QueryCursor};
use tree_sitter_json::language;

pub fn formatter(
    input_file: &Path,
    query_file: &Path,
    out: &mut dyn io::Write,
) -> Result<(), Box<dyn Error>> {
    // Read file
    let content = &fs::read_to_string(input_file)?;
    let query_str = &fs::read_to_string(query_file)?;

    // Parsing
    let json = language();

    let mut parser = Parser::new();
    parser
        .set_language(json)
        .expect("Error loading json grammar");
    let parsed = parser
        .parse(&content, None)
        .expect("Could not parse json file");
    let root = parsed.root_node();
    let source = content.as_bytes();
    let query = Query::new(json, query_str).expect("Error parsing query file");

    // Find the ids of all tree-sitter nodes that were identified as a leaf
    // We want to avoid recursing into them in the collect_leafs function.
    let specified_leaf_nodes: BTreeSet<usize> = collect_leaf_ids(&query, root, source);

    // Match queries
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root, source);

    // The Flattening: collects all terminal nodes of the tree-sitter tree in a Vec
    let mut atoms: Vec<Atom> = Vec::new();
    collect_leafs(root, &mut atoms, source, &specified_leaf_nodes);

    // Formatting
    for m in matches {
        for c in m.captures {
            let name = query.capture_names()[c.index as usize].clone();
            resolve_capture(name, &mut atoms, c.node);
        }
    }

    // Convert our list of atoms to a Doc
    let doc = atoms_to_doc(&mut 0, &atoms);
    doc.render(200, out)?;

    Ok(())
}

/// A Node from tree-sitter is turned into into a list of atoms
#[derive(Debug)]
enum Atom {
    Leaf { content: String, id: usize },
    Literal(String),
    Hardline,
    IndentEnd,
    IndentStart,
    Space,
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
) {
    if node.child_count() == 0 || specified_leaf_nodes.contains(&node.id()) {
        atoms.push(Atom::Leaf {
            content: String::from(node.utf8_text(source).expect("Source file not valid utf8")),
            id: node.id(),
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source, &specified_leaf_nodes)
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
                    atoms_to_doc(i, atoms).nest(4)
                }
                Atom::Space => RcDoc::space(),
            });
        }
        *i = *i + 1;
    }
    return doc;
}

fn resolve_capture(name: String, atoms: &mut Vec<Atom>, node: Node) {
    match name.as_ref() {
        "append_hardline" => atoms_append(Atom::Hardline, node, atoms),
        "append_space" => atoms_append(Atom::Space, node, atoms),
        "append_indent_start" => atoms_append(Atom::IndentStart, node, atoms),
        "prepend_indent_end" => atoms_prepend(Atom::IndentEnd, node, atoms),
        "prepend_indent_start" => atoms_prepend(Atom::IndentStart, node, atoms),
        "append_indent_end" => atoms_append(Atom::IndentEnd, node, atoms),
        "indented" => {
            atoms_prepend(Atom::IndentStart, node, atoms);
            atoms_append(Atom::IndentEnd, node, atoms);
        }
        "append_comma" => atoms_append(Atom::Literal(",".to_string()), node, atoms),
        // Skip over leafs
        _ => return,
    }
}

fn atoms_prepend(atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
    let target_node = first_leaf(node);
    let index = find_node(target_node, atoms);
    atoms.insert(index, atom);
}

fn atoms_append(atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
    let target_node = last_leaf(node);
    let index = find_node(target_node, atoms);
    if index > atoms.len() {
        atoms.push(atom);
    } else {
        atoms.insert(index + 1, atom);
    }
}
