use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use topiary_core::test_utils::pretty_assert_eq;

use tempfile::TempDir;

fn io_test(file: &str) {
    let input = PathBuf::from(format!("tests/samples/input/{file}"));
    let expected = PathBuf::from(format!("tests/samples/expected/{file}"));

    // Make sure our test makes sense
    assert!(input.exists() && expected.exists());

    // Load the known good formatted file
    let expected_output = fs::read_to_string(&expected).unwrap();

    // Stage the input to a temporary directory
    let tmp = TempDir::new().unwrap();
    let staged = tmp.path().join(file);
    fs::copy(input, &staged).unwrap();

    // Run Topiary against the staged input file
    let mut topiary = Command::cargo_bin("topiary").unwrap();
    let output = topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("fmt")
        .arg("-vvv")
        .arg(&staged)
        .output()
        .expect("Failed to run `topiary fmt`");

    // Print the invocation output, so that it can be inspected with --nocapture
    let output_str = String::from_utf8(output.stderr).expect("Failed to decode Topiary output");
    println!("{output_str}");

    // Read the file after formatting
    let formatted = fs::read_to_string(&staged).unwrap();

    // Assert the formatted file is as expected
    pretty_assert_eq(&expected_output, &formatted);
}

fn coverage_test(file: &str) {
    let input = PathBuf::from(format!("tests/samples/input/{file}"));

    // Make sure our test makes sense
    assert!(input.exists());

    // Run `topiary coverage` against the input file
    let mut topiary = Command::cargo_bin("topiary").unwrap();
    let output = topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries/")
        .arg("coverage")
        .arg(&input)
        .output()
        .expect("Failed to run `topiary coverage`");

    if !output.status.success() {
        let output_str = String::from_utf8(output.stdout).expect("Failed to decode topiary output");
        panic!("Insufficient coverage of \"{file}\":\n{output_str}")
    }
}

#[test]
fn input_output_tester() {
    // TODO There's probably a better way than this...

    #[cfg(feature = "bash")]
    io_test("bash.sh");

    #[cfg(feature = "css")]
    io_test("css.css");

    #[cfg(feature = "json")]
    io_test("json.json");

    #[cfg(feature = "nickel")]
    io_test("nickel.ncl");

    #[cfg(feature = "ocaml")]
    io_test("ocaml.ml");

    #[cfg(feature = "ocaml_interface")]
    io_test("ocaml-interface.mli");

    #[cfg(feature = "ocamllex")]
    io_test("ocamllex.mll");

    #[cfg(feature = "openscad")]
    io_test("openscad.scad");

    #[cfg(feature = "rust")]
    io_test("rust.rs");

    #[cfg(feature = "sdml")]
    io_test("sdml.sdml");

    #[cfg(feature = "toml")]
    io_test("toml.toml");

    #[cfg(feature = "tree_sitter_query")]
    io_test("tree_sitter_query.scm");
}

#[test]
fn coverage_tester() {
    // TODO There definitely should be a better way than this...

    #[cfg(feature = "bash")]
    coverage_test("bash.sh");

    #[cfg(feature = "css")]
    coverage_test("css.css");

    #[cfg(feature = "json")]
    coverage_test("json.json");

    #[cfg(feature = "nickel")]
    coverage_test("nickel.ncl");

    #[cfg(feature = "ocaml")]
    coverage_test("ocaml.ml");

    // "ocaml-interface.mli" is voluntarily omitted:
    // the queries should all be covered by "ocaml.ml"

    #[cfg(feature = "ocamllex")]
    coverage_test("ocamllex.mll");

    #[cfg(feature = "rust")]
    coverage_test("rust.rs");

    #[cfg(feature = "sdml")]
    coverage_test("sdml.sdml");

    #[cfg(feature = "toml")]
    coverage_test("toml.toml");

    #[cfg(feature = "tree_sitter_query")]
    coverage_test("tree_sitter_query.scm");
}

// Test that our query files are properly formatted
#[test]
#[cfg(feature = "tree_sitter_query")]
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
