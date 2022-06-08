use pretty::RcDoc;
use std::{error::Error, fs};
use tree_sitter::{Node, Parser, Query, QueryCursor};
use tree_sitter_json::language;

static TEST_FILE: &str = "tests/json.json";
static QUERY_FILE: &str = "languages/queries/json.scm";

/// A Node from tree-sitter is turned into into a list of atoms
#[derive(Debug)]
enum Atom {
    Leaf { content: String, id: usize },
    Literal(String),
    Hardline,
    IndentEnd,
    IndentStart,
    Softline,
    Space,
}

/// Given a node, returns the id of the first leaf in the subtree.
fn first_leaf_id(node: Node) -> usize {
    if node.child_count() == 0 {
        node.id()
    } else {
        first_leaf_id(node.child(0).unwrap())
    }
}

/// Given a node, returns the id of the last leaf in the subtree.
fn last_leaf_id(node: Node) -> usize {
    let nr_children = node.child_count();
    if nr_children == 0 {
        node.id()
    } else {
        last_leaf_id(node.child(nr_children - 1).unwrap())
    }
}

fn collect_leafs<'a>(node: Node, atoms: &mut Vec<Atom>, source: &'a [u8]) {
    if node.child_count() == 0 {
        atoms.push(Atom::Leaf {
            content: String::from(node.utf8_text(source).expect("Source file not valid utf8")),
            id: node.id(),
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source)
        }
    }
}

/// Finds the matching node in the atoms and returns the index
/// TODO: Error
fn find_node(wanted_id: usize, atoms: &mut Vec<Atom>) -> usize {
    for (i, node) in atoms.iter().enumerate() {
        match node {
            Atom::Leaf { id, .. } => {
                if *id == wanted_id {
                    return i
                }
            },
            _ => continue,
        }
    }
    return 0;
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read file
    let content = &fs::read_to_string(TEST_FILE)?;
    let query_str = &fs::read_to_string(QUERY_FILE)?;

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

    // The Flattening
    let mut atoms: Vec<Atom> = Vec::new();
    collect_leafs(root, &mut atoms, source);

    // Match queries
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root, source);

    // Formatting
    for m in matches {
        for c in m.captures {
            let name = query.capture_names()[c.index as usize].clone();
            resolve_capture(name, &mut atoms, c.node);
        }
    }

    // Printing
    atoms_print(atoms);

    Ok(())
}

fn atoms_print(atoms: Vec<Atom>) {
    for a in atoms {
        match a {
            Atom::Leaf { content, .. } => print!("{}", content),
            Atom::Literal(s) => print!("{}", s),
            Atom::Hardline => print!("\n"),
            Atom::IndentEnd => todo!(),
            Atom::IndentStart => todo!(),
            Atom::Softline => print!("\n"),
            Atom::Space => print!(" "),
        }
    }
}

fn resolve_capture(name: String, atoms: &mut Vec<Atom>, node: Node) -> () {
    match name.as_ref() {
        "append_hardline" => atoms_append(Atom::Hardline, node, atoms),
        "append_space" => atoms_append(Atom::Space, node, atoms),
        "indented" => {
            atoms_append(Atom::IndentStart, node, atoms);
            atoms_append(Atom::IndentEnd, node, atoms);
        },
        "append_comma" => atoms_append(Atom::Literal(",".to_string()), node, atoms),
        _ => return,
    }
}

fn atoms_prepend(atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
    let id = first_leaf_id(node);
    let index = find_node(id, atoms);
    atoms.insert(index, atom);
}

fn atoms_append(atom: Atom, node: Node, atoms: &mut Vec<Atom>) {
    let id = last_leaf_id(node);
    let index = find_node(id, atoms);
    atoms.insert(index + 1, atom);
}
