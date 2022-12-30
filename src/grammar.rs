//! This module deals with the tree-sitter grammars. For the purpose of
//! Topiary we consider grammars to be the entire source code of a tree-sitter
//! grammar project. This module is responsible for fetching and building them
//! to make them available for the formatter portion of Topiary.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{Oid, Repository};
use serde_derive::{Deserialize, Serialize};

use crate::{
    error::ParserCompilationError, project_dirs::TOPIARY_DIRS, FormatterError, FormatterResult,
};

const BUILD_TARGET: &str = env!("BUILD_TARGET");

#[cfg(unix)]
pub const DYLIB_EXTENSION: &str = "so";

#[cfg(windows)]
pub const DYLIB_EXTENSION: &str = "dll";

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
                let grammar_path = fetch_grammar(remote, revision)?;
                compile_grammar(&grammar_path, name);
                Ok(())
            }
        }
    }
}

/// Fetches the grammar from the remote, returns the location of the grammar.
fn fetch_grammar(remote: &str, revision: &str) -> FormatterResult<PathBuf> {
    let mut path = TOPIARY_DIRS.cache_dir().to_path_buf();
    path.push("grammars/");
    path.push(format!("{}/", revision));

    // Try to open, if fails, create new.
    let repo = match Repository::open(&path) {
        // If it can succesfully open the repository, we can safely return
        Ok(_) => return Ok(path),
        Err(_) => match Repository::clone_recurse(remote, &path) {
            Ok(repo) => repo,
            Err(e) => return Err(FormatterError::Git(e)),
        },
    };

    // Set detached head to the required revision
    let oid = Oid::from_str(revision).map_err(|e| FormatterError::Git(e))?;
    repo.set_head_detached(oid)
        .map_err(|e| FormatterError::Git(e))?;
    Ok(path)
}

// Since we do not know if a grammar is already built or not, we always build
// it. We could possible avoid this by tagging the binary with some kind of
// revision.
fn compile_grammar(source_path: &Path, name: &str) -> FormatterResult<()> {
    let src_path = source_path.join("src");
    let header_path = src_path.clone();
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

    let mut library_path = TOPIARY_DIRS.cache_dir().join(format!("parsers/{}/", name));

    // Ensure path exists
    fs::create_dir_all(&library_path)
        .map_err(|e| FormatterError::ParserCompilation(ParserCompilationError::Io(e)))?;

    library_path.push("parser");
    library_path.set_extension(DYLIB_EXTENSION);

    let mut config = cc::Build::new();
    config
        .cpp(true)
        .opt_level(3)
        .cargo_metadata(false)
        .host(BUILD_TARGET)
        .target(BUILD_TARGET);
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

    let output = command
        .output()
        .map_err(|e| FormatterError::ParserCompilation(ParserCompilationError::Io(e)))?;

    if !output.status.success() {
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);

        return Err(FormatterError::ParserCompilation(
            ParserCompilationError::Cc(out.into_owned(), err.into_owned()),
        ));
    }

    Ok(())
}
