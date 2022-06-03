use pretty::RcDoc;
use std::{error::Error, fs, io};
use tree_sitter::{Node, Parser, Query, QueryCursor};
use tree_sitter_json::language;

static TEST_FILE: &str = "tests/json.json";
static QUERY_FILE: &str = "languages/queries/json.scm";

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

    for m in matches {
        for c in m.captures {
            println!("{:#?}", c.node.kind());
        }
    }

    Ok(())
}
