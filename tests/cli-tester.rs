use assert_cmd::Command;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

// Exemplar JSON state that won't be affected by the formatter
const STATE: &str = "\"test\"";

fn create_state() -> PathBuf {
    let mut json = NamedTempFile::new().unwrap();
    write!(json, "{STATE}").unwrap();

    json.keep().unwrap().1
}

fn read_state(path: &PathBuf) -> String {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    contents
}

#[test]
fn test_no_clobber() {
    let input_path = create_state();

    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .arg("--language")
        .arg("json")
        .arg("--input-file")
        .arg(&input_path)
        .arg("--output-file")
        .arg(&input_path)
        .assert()
        .success();

    let output = read_state(&input_path);
    assert_eq!(output.trim(), STATE);
}

#[test]
fn test_in_place() {
    let input_path = create_state();

    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .arg("--language")
        .arg("json")
        .arg("--input-file")
        .arg(&input_path)
        .arg("--in-place")
        .assert()
        .success();

    let output = read_state(&input_path);
    assert_eq!(output.trim(), STATE);
}

#[test]
fn test_in_place_no_input() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .arg("--language")
        .arg("json")
        .arg("--in-place")
        .assert()
        .failure();
}
