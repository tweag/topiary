//! Integration tests for shebang-based language detection.

use std::fs;
use topiary_config::Configuration;

#[test]
fn test_bash_shebang_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/bin/bash\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "bash");
}

#[test]
fn test_zsh_shebang_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/bin/zsh\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "zsh");
}

#[test]
fn test_bash_env_shebang_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/usr/bin/env bash\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "bash");
}

#[test]
fn test_zsh_env_shebang_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/usr/bin/env zsh\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "zsh");
}

#[test]
fn test_bash_extension_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.bash");
    fs::write(&file_path, "echo hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "bash");
}

#[test]
fn test_zsh_extension_detection() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.zsh");
    fs::write(&file_path, "echo hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "zsh");
}

#[test]
fn test_no_shebang_defaults_to_match() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "echo hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    // Without shebang, should return one of the languages that matches .sh extension
    // The specific language depends on internal ordering, but it should be bash or zsh
    assert!(
        lang.name == "bash" || lang.name == "zsh",
        "Expected bash or zsh, got: {}",
        lang.name
    );
}

#[test]
fn test_env_with_flags() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/usr/bin/env -S bash\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "bash");
}

#[test]
fn test_shebang_with_args() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.sh");
    fs::write(&file_path, "#!/bin/zsh -f\necho hello").unwrap();

    let config = Configuration::default();
    let lang = config.detect(&file_path).unwrap();

    assert_eq!(lang.name, "zsh");
}
