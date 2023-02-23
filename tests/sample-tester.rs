use pretty_assertions::assert_eq;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use test_log::test;
use topiary::{formatter, Language};

#[test]
fn input_output_tester() {
    let input_dir = fs::read_dir("tests/samples/input").unwrap();
    let expected_dir = Path::new("tests/samples/expected");

    for file in input_dir {
        let file = file.unwrap();

        let input_path = file.path();
        let language: Language = input_path.try_into().unwrap();

        let expected_path = expected_dir.join(file.file_name());
        let expected = fs::read_to_string(expected_path).unwrap();

        let mut input = BufReader::new(fs::File::open(file.path()).unwrap());
        let mut output = Vec::new();
        let query = fs::read_to_string(PathBuf::try_from(language).unwrap()).unwrap();
        let mut query = query.as_bytes();

        formatter(&mut input, &mut output, &mut query, Some(language), true).unwrap();
        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{}", formatted);

        assert_eq!(expected, formatted);
    }
}

// Test that our query files are properly formatted
#[test]
fn formatted_query_tester() {
    let language_dir = fs::read_dir("languages").unwrap();

    for file in language_dir {
        let file = file.unwrap();
        let language = Language::TreeSitterQuery;

        let expected = fs::read_to_string(file.path()).unwrap();

        let mut input = BufReader::new(fs::File::open(file.path()).unwrap());
        let mut output = Vec::new();
        let query = fs::read_to_string(PathBuf::try_from(language).unwrap()).unwrap();
        let mut query = query.as_bytes();

        formatter(&mut input, &mut output, &mut query, Some(language), true).unwrap();
        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{}", formatted);

        assert_eq!(expected, formatted);
    }
}
