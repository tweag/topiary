use pretty::RcDoc;
use std::{error::Error, fs, io};
use tree_sitter::{Node, Parser, Query, QueryCursor, Range};
use tree_sitter_json::language;

static TEST_FILE: &str = "tests/json.json";
static QUERY_FILE: &str = "languages/queries/json.scm";

/// A Node from tree-sitter is turned into into a list of atoms
#[derive(Debug)]
enum Atom {
    Leaf { content: String, range: Range },
    Softline,
    Hardline,
    IndentStart,
    IndentEnd,
}

//
fn collect_leafs<'a>(node: Node, atoms: &mut Vec<Atom>, source: &'a [u8]) {
    if node.child_count() == 0 {
        atoms.push(Atom::Leaf {
            content: String::from(node.utf8_text(source).unwrap()),
            range: node.range(),
        });
    } else {
        for child in node.children(&mut node.walk()) {
            collect_leafs(child, atoms, source)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read file
    let content = &fs::read_to_string(TEST_FILE)?;
    let query_str = &fs::read_to_string(QUERY_FILE)?;

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

    let mut cursor = QueryCursor::new();

    let matches = cursor.matches(&query, root, source);

    let mut atoms: Vec<Atom> = Vec::new();

    collect_leafs(root, &mut atoms, source);

    println!("{:#?}", atoms);

    Ok(())
}
