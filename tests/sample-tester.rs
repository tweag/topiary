use std::fs;
use std::io::BufWriter;
use test_log::test;
use tree_sitter_formatter::formatter;
use tree_sitter_formatter::Language;

#[test]
fn sample_tester() {
    let dir = fs::read_dir("tests/samples").unwrap();

    for file in dir {
        let path = file.unwrap().path();
        let extension = path.extension().unwrap().to_str().unwrap();

        let language = match extension {
            "json" => Language::Json,
            "rs" => Language::Rust,
            _ => panic!("File extension {} not supported.", extension),
        };

        let sample = fs::read_to_string(path).unwrap();
        let mut input = sample.as_bytes();
        let mut output = BufWriter::new(Vec::new());
        formatter(&mut input, &mut output, language);
        let bytes = output.into_inner().unwrap();
        let formatted = String::from_utf8(bytes).unwrap();
        log::debug!("{}", formatted);

        assert_eq!(sample, formatted);
    }
}
