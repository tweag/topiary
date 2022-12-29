//! This module deals with the tree-sitter grammars.
//! It is responsible for fetching and building them.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use git2::{Oid, Repository};
use serde_derive::{Deserialize, Serialize};

use crate::{project_dirs::TOPIARY_DIRS, FormatterResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum GrammarSource {
    Local {
        path: String,
    },
    Git {
        #[serde(rename = "git")]
        remote: String,
        #[serde(rename = "rev")]
        revision: String,
    },
}

impl GrammarSource {
    /// Ensures the grammar is updated and available. This includes fetching,
    /// compiling and placing it in the required place.
    pub fn ensure_available(&self, name: &str) -> FormatterResult<()> {
        match self {
            GrammarSource::Local { path } => {
                let path = Path::new(path);
                if !path.exists() {
                    // Return error
                    todo!()
                } else {
                    compile_grammar(path, name);
                }
                Ok(())
            }
            GrammarSource::Git { remote, revision } => {
                // We could construct the path ourselves
                let grammar_path = fetch_grammar(remote, revision);
                compile_grammar(&grammar_path, name);
                Ok(())
            }
        }
    }
}

/// Fetches the grammar from the remote, returns doing nothing if already present.
fn fetch_grammar(remote: &str, revision: &str) -> PathBuf {
    let mut path = TOPIARY_DIRS.cache_dir().to_path_buf();
    path.push("grammars/");
    path.push(format!("{}/", revision));

    // Try to open, if fails, create new.
    let repo = match Repository::open(&path) {
        // If it can succesfully open the repository, we can safely return
        Ok(repo) => return path,
        Err(e) => match Repository::clone_recurse(remote, &path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        },
    };

    // Set detached head to the required revision
    // TODO: error
    repo.set_head_detached(Oid::from_str(revision).unwrap())
        .unwrap();
    return path;
}

// Since we do not know if a grammar is already built or not, we always build
// it. We could possible avoid this by tagging the binary with some kind of
// revision.
fn compile_grammar(src_path: &Path, name: &str) {
    let header_path = src_path;
    let parser_path = src_path.join("parser.c");
    let mut scanner_path = src_path.join("scanner.c");

    let scanner_path = if scanner_path.exists() {
        Some(scanner_path)
    } else {
        scanner_path.set_extension("cc");
        if scanner_path.exists() {
            Some(scanner_path)
        } else {
            None
        }
    };

    let mut library_path = TOPIARY_DIRS
        .cache_dir()
        .join(format!("parsers/{}/parser", name));
    // TODO: Not assume Linux
    library_path.set_extension(".so");

    let mut config = cc::Build::new();
    config.cpp(true).opt_level(3).cargo_metadata(false);
    let compiler = config.get_compiler();
    let mut command = Command::new(compiler.path());
    command.current_dir(src_path);
    for (key, value) in compiler.env() {
        command.env(key, value);
    }
    command.args(compiler.args());

    command
        .arg("-shared")
        .arg("-fPIC")
        .arg("-fno-exceptions")
        .arg("-g")
        .arg("-I")
        .arg(header_path)
        .arg("-o")
        .arg(&library_path)
        .arg("-O3");
    if let Some(scanner_path) = scanner_path.as_ref() {
        if scanner_path.extension() == Some("c".as_ref()) {
            command.arg("-xc").arg("-std=c99").arg(scanner_path);
        } else {
            command.arg(scanner_path);
        }
    }
    command.arg("-xc").arg(parser_path);
    command.arg("-Wl,-z,relro,-z,now");

    let output = command.output().unwrap();
    if !output.status.success() {
        todo!("error")
    }
}
