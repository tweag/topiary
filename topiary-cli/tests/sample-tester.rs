use assert_cmd::Command;
use std::fs;
use std::path::Path;
use topiary_core::test_utils::pretty_assert_eq;

use tempfile::TempDir;

#[test]
fn input_output_tester() {
    let input_dir = fs::read_dir("tests/samples/input").unwrap();
    let expected_dir = Path::new("tests/samples/expected");

    for file in input_dir {
        let file = file.unwrap();

        // Load the known good formated files
        let expected_path = expected_dir.join(file.file_name());
        let expected = fs::read_to_string(expected_path).unwrap();

        let tmp_dir = TempDir::new().unwrap();

        // Copy the file to a temp dir
        let mut input_file = tmp_dir.path().to_path_buf();
        input_file.push(file.path().file_name().unwrap());
        fs::copy(file.path(), &input_file).unwrap();

        // Run topiary on the input file in the temp dir
        let mut topiary = Command::cargo_bin("topiary").unwrap();
        topiary
            .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
            .arg("fmt")
            .arg(&input_file)
            .assert()
            .success();

        // Read the file after formatting
        let formatted = fs::read_to_string(input_file).unwrap();

        // Assert the formatted file is as expected
        pretty_assert_eq(&expected, &formatted);
    }
}

// Test that our query files are properly formatted
#[test]
fn formatted_query_tester() {
    let language_dir = fs::read_dir("../topiary-queries/queries").unwrap();

    for file in language_dir {
        let file = file.unwrap();

        // Load the query file (we assume is formatted correctly)
        let expected = fs::read_to_string(file.path()).unwrap();

        let tmp_dir = TempDir::new().unwrap();

        // Copy the file to a temp dir
        let mut input_file = tmp_dir.path().to_path_buf();
        input_file.push(file.path().file_name().unwrap());
        fs::copy(file.path(), &input_file).unwrap();

        // Run topiary on the input file in the temp dir
        let mut topiary = Command::cargo_bin("topiary").unwrap();
        topiary
            .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
            .arg("fmt")
            .arg(&input_file)
            .assert()
            .success();

        // Read the file after formatting
        let formatted = fs::read_to_string(input_file).unwrap();

        pretty_assert_eq(&expected, &formatted);
    }
}
