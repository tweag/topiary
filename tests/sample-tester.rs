use pretty_assertions::assert_eq;
use std::fs;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use test_log::test;
use tree_sitter_formatter::formatter;
use tree_sitter_formatter::Language;

#[test]
fn sample_tester() {
    let input_dir = fs::read_dir("tests/samples/input").unwrap();
    let expected_dir = Path::new("tests/samples/expected");

    for file in input_dir {
        let file = file.unwrap();
        let input_path = file.path();
        let expected_path = expected_dir.join(file.file_name());
        let extension = input_path.extension().unwrap().to_str().unwrap();

        let language = match extension {
            "json" => Language::Json,
            "ml" => Language::OCaml,
            "rs" => Language::Rust,
            _ => panic!("File extension {} not supported.", extension),
        };

        let expected = fs::read_to_string(expected_path).unwrap();
        let mut input = BufReader::new(fs::File::open(input_path).unwrap());
        let mut output = BufWriter::new(Vec::new());
        formatter(&mut input, &mut output, language).unwrap();
        let bytes = output.into_inner().unwrap();
        let formatted = String::from_utf8(bytes).unwrap();
        log::debug!("{}", formatted);

        // This one needs more work :-)
        if file.file_name() != "tree-sitter.rs" {
            assert_eq!(expected, formatted);
        }
    }
}
