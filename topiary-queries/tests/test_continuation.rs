use topiary_queries::bash;
use topiary_tree_sitter_facade::{Node, Parser};

fn inspect_node(node: Node, source: &[u8], indent: usize) {
    let indent_str = "  ".repeat(indent);
    let text = node.utf8_text(source).unwrap();
    let byte_range = node.byte_range();

    // Extract the actual bytes for this node
    let node_bytes = &source[byte_range.start as usize..byte_range.end as usize];
    let contains_backslash = node_bytes.iter().any(|&b| b == b'\\');

    if contains_backslash {
        println!("{}✓ Node '{}' contains backslash:", indent_str, node.kind());
        println!("{}  Text: {:?}", indent_str, text);
        println!(
            "{}  Bytes: {:?}",
            indent_str,
            std::str::from_utf8(node_bytes).unwrap_or("<invalid utf8>")
        );
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        inspect_node(child, source, indent + 1);
    }
}

#[test]
fn test_line_continuation_detection() {
    let source = r#"#!/usr/bin/env bash

# Simple line continuation test
echo foo \
  bar

# Another test
baz \
&& quux
"#;

    let mut parser = Parser::new(bash()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    println!("\n=== Analyzing parse tree for backslashes ===\n");
    inspect_node(root, source.as_bytes(), 0);
}
