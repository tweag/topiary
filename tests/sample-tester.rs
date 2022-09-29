use pretty_assertions::assert_eq;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use test_log::test;
use tree_sitter_formatter::{formatter, Language};

#[test]
fn input_output_tester() {
    let input_dir = fs::read_dir("tests/samples/input").unwrap();
    let expected_dir = Path::new("tests/samples/expected");

    for file in input_dir {
        let file = file.unwrap();
        let input_path = file.path();
        let expected_path = expected_dir.join(file.file_name());
        let extension = input_path.extension().unwrap().to_str().unwrap();

        let language = match extension {
            "json" => Language::Json,
            "ml" => Language::Ocaml,
            "rs" => Language::Rust,
            "toml" => Language::Toml,
            _ => panic!("File extension {} not supported.", extension),
        };

        let expected = fs::read_to_string(expected_path).unwrap();
        let mut input = BufReader::new(fs::File::open(input_path).unwrap());
        let mut output = Vec::new();
        let query_path = str::to_lowercase(format!("languages/{language:?}.scm").as_str());
        let query = fs::read_to_string(query_path).unwrap();
        let mut query = query.as_bytes();

        formatter(&mut input, &mut output, &mut query, true).unwrap();
        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{}", formatted);

        assert_eq!(expected, formatted);
    }
}
