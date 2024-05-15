use std::{fmt, fs, fs::File, io::Write, path::PathBuf};

use assert_cmd::Command;
use predicates::{
    prelude::PredicateBooleanExt,
    str::{ends_with, starts_with},
};
use tempfile::TempDir;

// Simple exemplar JSON and TOML state, to verify the formatter
// is doing something... and hopefully the right thing
const JSON_INPUT: &str = r#"{   "test"  :123}"#;
const JSON_EXPECTED: &str = r#"{ "test": 123 }
"#;

const TOML_INPUT: &str = r#"   test=    123"#;
const TOML_EXPECTED: &str = r#"test = 123
"#;

// The TempDir member of the State is not actually used.
// However, removing it means that the directory is dropped at the end of the new() function, which causes it to be deleted.
// This causes the path to the state file to be invalid and breaks the tests.
// So, we keep the TempDir around so the tests don't break.
#[allow(dead_code)]
struct State(TempDir, PathBuf);

impl State {
    fn new(payload: &str, extension: &str) -> Self {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_file = tmp_dir.path().join(format!("state.{extension}"));

        let mut state = File::create(&tmp_file).unwrap();
        write!(state, "{payload}").unwrap();

        Self(tmp_dir, tmp_file)
    }

    fn path(&self) -> &PathBuf {
        &self.1
    }

    fn read(&self) -> String {
        fs::read_to_string(self.path()).unwrap()
    }
}

#[test]
fn test_fmt_stdin() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
}

#[test]
fn test_fmt_stdin_query() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("--query")
        .arg("../topiary-queries/queries/json.scm")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(JSON_EXPECTED);
}

#[test]
fn test_fmt_files() {
    let json = State::new(JSON_INPUT, "json");
    let toml = State::new(TOML_INPUT, "toml");

    let mut topiary = Command::cargo_bin("topiary").unwrap();

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg(json.path())
        .arg(toml.path())
        .assert()
        .success();

    assert_eq!(json.read(), JSON_EXPECTED);
    assert_eq!(toml.read(), TOML_EXPECTED);
}

#[test]
fn test_fmt_dir() {
    let json = State::new(JSON_INPUT, "json");

    let mut topiary = Command::cargo_bin("topiary").unwrap();

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg(json.path().parent().unwrap())
        .assert()
        .success();

    assert_eq!(json.read(), JSON_EXPECTED);
}

#[test]
fn test_fmt_invalid() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    // Can't specify --language with input files
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("fmt")
        .arg("--language")
        .arg("json")
        .arg("/path/to/some/input")
        .assert()
        .failure();

    // Can't specify --query without --language
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../whatever")
        .arg("fmt")
        .arg("--query")
        .arg("/path/to/query")
        .assert()
        .failure();
}

#[test]
fn test_vis() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    // Sanity check output is a valid DOT graph
    let is_graph = starts_with("graph {").and(ends_with("}\n"));

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--language")
        .arg("json")
        .write_stdin(JSON_INPUT)
        .assert()
        .success()
        .stdout(is_graph);
}

#[test]
fn test_vis_invalid() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    // Can't specify --language with input file
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--language")
        .arg("json")
        .arg("/path/to/some/input")
        .assert()
        .failure();

    // Can't specify --query without --language
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("--query")
        .arg("/path/to/query")
        .assert()
        .failure();

    // Can't specify multiple input files
    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("vis")
        .arg("/path/to/some/input")
        .arg("/path/to/another/input")
        .assert()
        .failure();
}

#[test]
fn test_cfg() {
    let mut topiary = Command::cargo_bin("topiary").unwrap();

    topiary
        .env("TOPIARY_LANGUAGE_DIR", "../topiary-queries/queries")
        .arg("cfg")
        .assert()
        .success()
        .stdout(IsToml);
}

struct IsToml;

impl predicates::Predicate<str> for IsToml {
    fn eval(&self, variable: &str) -> bool {
        toml::Value::try_from(variable).is_ok()
    }
}

impl predicates::reflection::PredicateReflection for IsToml {}

impl fmt::Display for IsToml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "is_toml")
    }
}
