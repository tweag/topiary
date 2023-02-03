use assert_cmd::Command;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tempfile::NamedTempFile;

// Exemplar JSON state that won't be affected by the formatter
const JSON: &str = "\"test\"";

struct State(NamedTempFile);

impl State {
    fn new(payload: &str) -> Self {
        let mut state = NamedTempFile::new().unwrap();
        write!(state, "{payload}").unwrap();

        Self(state)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn read(&self) -> String {
        // For an in place edit, Topiary will remove the original file. As such, we can't use
        // NamedTempFile::reopen, as the original no longer exists; we have to "reopen" it by path.
        let mut file = File::open(self.path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        contents
    }
}

#[test]
fn test_file_output() {
    let output = State::new("");

    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "languages")
        .arg("--language")
        .arg("json")
        .arg("--output-file")
        .arg(output.path())
        .write_stdin(JSON)
        .assert()
        .success();

    assert_eq!(output.read().trim(), JSON);
}

#[test]
fn test_no_clobber() {
    let json = State::new(JSON);
    let input_path = json.path();

    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "languages")
        .arg("--language")
        .arg("json")
        .arg("--input-file")
        .arg(&input_path)
        .arg("--output-file")
        .arg(&input_path)
        .assert()
        .success();

    // NOTE We only assume, here, that the state has been modified by the call to Topiary. It may
    // be worthwhile asserting (e.g., change in mtime, etc.).
    assert_eq!(json.read().trim(), JSON);
}

#[test]
fn test_in_place() {
    let json = State::new(JSON);
    let input_path = json.path();

    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "languages")
        .arg("--language")
        .arg("json")
        .arg("--input-file")
        .arg(&input_path)
        .arg("--in-place")
        .assert()
        .success();

    // NOTE We only assume, here, that the state has been modified by the call to Topiary. It may
    // be worthwhile asserting (e.g., change in mtime, etc.).
    assert_eq!(json.read().trim(), JSON);
}

#[test]
fn test_in_place_no_input() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "languages")
        .arg("--language")
        .arg("json")
        .arg("--in-place")
        .assert()
        .failure();
}
