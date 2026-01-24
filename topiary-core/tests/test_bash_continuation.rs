#[test]
fn test_bash_command_node_text_with_continuation() {
    use tree_sitter::{Language, Parser};

    unsafe extern "C" {
        fn tree_sitter_bash() -> Language;
    }

    let source = "echo foo \\\n  bar";

    let mut parser = Parser::new();
    unsafe {
        parser.set_language(&tree_sitter_bash()).unwrap();
    }

    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();

    println!("\n=== Source ===");
    println!("{:?}", source);
    println!("Bytes: {:?}", source.as_bytes());

    // Find the command node
    let command = root.child(0).expect("Should have a child");

    println!("\n=== Command Node ===");
    println!("Kind: {}", command.kind());
    println!("Byte range: {:?}", command.byte_range());

    let command_text = command.utf8_text(source.as_bytes()).unwrap();
    println!("utf8_text(): {:?}", command_text);
    println!("utf8_text() bytes: {:?}", command_text.as_bytes());

    let byte_range = command.byte_range();
    let range_bytes = &source.as_bytes()[byte_range];
    println!("Direct byte slice: {:?}", range_bytes);
    println!(
        "Direct byte slice as str: {:?}",
        std::str::from_utf8(range_bytes).unwrap()
    );

    println!("\n=== Comparison ===");
    println!(
        "utf8_text contains backslash: {}",
        command_text.contains('\\')
    );
    println!(
        "Byte range contains backslash: {}",
        range_bytes.contains(&b'\\')
    );

    assert!(
        command_text.contains('\\'),
        "Command text should contain backslash"
    );
    assert!(
        command_text.contains('\n'),
        "Command text should contain newline"
    );
}
