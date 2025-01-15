//! This module contains the `Language` struct, which represents a language configuration, and
//! associated methods.

use crate::error::TopiaryConfigResult;
#[cfg(not(target_arch = "wasm32"))]
use crate::error::{TopiaryConfigError, TopiaryConfigFetchingError};
use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
const BUILD_TARGET: &str = env!("BUILD_TARGET");

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in the deserialised
    /// configuration and to convert to the respective Tree-sitter grammar
    pub name: String,

    /// The configuration of the language, includes all properties that Topiary
    /// needs to properly format the language
    pub config: LanguageConfiguration,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct LanguageConfiguration {
    /// A set of the filetype extensions associated with this language. This enables Topiary to
    /// switch to the right language based on the input filename.
    pub extensions: HashSet<String>,

    /// The indentation string used for this language; defaults to "  " (i.e., two spaces). Any
    /// string can be provided, but in most instances it will be some whitespace (e.g., "    ",
    /// "\t", etc.)
    pub indent: Option<String>,

    /// The tree-sitter source of the language, contains all that is needed to pull and compile the tree-sitter grammar
    pub grammar: Grammar,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct Grammar {
    #[cfg(not(target_arch = "wasm32"))]
    pub source: GrammarSource,
    /// If symbol of the language in the compiled grammar. Usually this is
    /// `tree_sitter_<LANGUAGE_NAME>`, but in rare cases it differs. For
    /// instance our "tree-sitter-query" language, where the symbol is:
    /// `tree_sitter_query` instead of `tree_sitter_tree_sitter_query`.
    pub symbol: Option<String>,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub enum GrammarSource {
    #[serde(rename = "git")]
    Git(GitSource),
    #[serde(rename = "path")]
    Path(PathBuf),
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct GitSource {
    /// The URL of the git repository that contains the tree-sitter grammar.
    pub git: String,
    /// The revision of the git repository to use.
    pub rev: String,
    /// The sub-directory within the repository where the grammar is located. Defaults to the root of the repository
    pub subdir: Option<String>,
}

impl Language {
    pub fn new(name: String, config: LanguageConfiguration) -> Self {
        Self { name, config }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_query_file(&self) -> TopiaryConfigResult<PathBuf> {
        let basename = PathBuf::from(self.name.as_str()).with_extension("scm");

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 4] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Some(PathBuf::from("./topiary-queries/queries")),
            Some(PathBuf::from("../topiary-queries/queries")),
        ];

        potentials
            .into_iter()
            .flatten()
            .map(|path| path.join(&basename))
            .find(|path| path.exists())
            .ok_or_else(|| TopiaryConfigError::QueryFileNotFound(basename))
    }

    #[cfg(not(target_arch = "wasm32"))]
    // Returns the library path, and ensures the parent directories exist.
    pub fn library_path(&self) -> std::io::Result<PathBuf> {
        match &self.config.grammar.source {
            GrammarSource::Git(git_source) => {
                let mut library_path = crate::project_dirs().cache_dir().to_path_buf();
                library_path.push(self.name.clone());
                std::fs::create_dir_all(&library_path)?;

                // Set the output path as the revision of the grammar
                library_path.push(git_source.rev.clone());

                // TODO: Windows Support
                // On both MacOS and Linux, .so is a valid file extension for shared objects.
                library_path.set_extension("so");

                Ok(library_path)
            }
            GrammarSource::Path(path) => Ok(path.to_path_buf()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    // NOTE: Much of the following code is heavily inspired by the `helix-loader` crate with license MPL-2.0.
    // To be safe, assume any and all of the following code is MLP-2.0 and copyrighted to the Helix project.
    pub fn grammar(
        &self,
    ) -> Result<topiary_tree_sitter_facade::Language, TopiaryConfigFetchingError> {
        let library_path = self.library_path()?;

        // Ensure the comile exists
        if !library_path.is_file() {
            match &self.config.grammar.source {
                GrammarSource::Git(git_source) => {
                    git_source.fetch_and_compile(&self.name, library_path.clone())?
                }
                GrammarSource::Path(_) => {
                    return Err(TopiaryConfigFetchingError::GrammarFileNotFound(
                        library_path,
                    ))
                }
            }
        }

        assert!(library_path.is_file());
        log::debug!("Loading grammar from {}", library_path.to_string_lossy());

        use libloading::{Library, Symbol};

        let library = unsafe { Library::new(&library_path) }?;
        let language_fn_name = if let Some(symbol_name) = self.config.grammar.symbol.clone() {
            symbol_name
        } else {
            format!("tree_sitter_{}", self.name.replace('-', "_"))
        };

        let language = unsafe {
            let language_fn: Symbol<unsafe extern "C" fn() -> *const ()> =
                library.get(language_fn_name.as_bytes())?;
            tree_sitter_language::LanguageFn::from_raw(*language_fn)
        };
        std::mem::forget(library);
        Ok(topiary_tree_sitter_facade::Language::from(language))
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
        let language_name = self.name.as_str();

        let grammar_path = if language_name == "tree_sitter_query" {
            "/playground/scripts/tree-sitter-query.wasm".to_string()
        } else {
            format!("/playground/scripts/tree-sitter-{language_name}.wasm")
        };

        Ok(
            topiary_web_tree_sitter_sys::Language::load_path(&grammar_path)
                .await
                .map_err(|e| {
                    let error: topiary_tree_sitter_facade::LanguageError = e.into();
                    error
                })?
                .into(),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl GitSource {
    fn fetch_and_compile(
        &self,
        name: &str,
        library_path: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        log::info!(
            "{}: Language Grammar not found, attempting to fetch and compile it",
            name
        );
        // Create a temporary directory to clone the repository to. We could
        // cached the repositories, but the additional disk space is probably
        // not worth the benefits gained by caching. The tempdir is deleted
        // when dropped
        let tmp_dir = tempfile::tempdir()?;

        self.fetch_and_compile_with_dir(name, library_path, tmp_dir.into_path())
    }

    pub fn fetch_and_compile_with_dir(
        &self,
        name: &str,
        library_path: PathBuf,
        tmp_dir: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        if library_path.is_file() {
            log::info!("{}: Built grammar already exists; nothing to do", name);
            return Ok(());
        }
        let tmp_dir = tmp_dir.join(name);

        // Clone the repository and checkout the configured revision
        log::info!("{}: cloning from {}", name, self.git);
        Command::new("git")
            .arg("clone")
            .arg(&self.git)
            .arg(&tmp_dir)
            .status()
            .map_err(TopiaryConfigFetchingError::Git)?;
        log::info!("{}: checking out {}", name, self.rev);
        let current_dir = env::current_dir().map_err(TopiaryConfigFetchingError::Io)?;
        env::set_current_dir(&tmp_dir).map_err(TopiaryConfigFetchingError::Io)?;
        Command::new("git")
            .arg("checkout")
            .arg(&self.rev)
            .status()
            .map_err(TopiaryConfigFetchingError::Git)?;
        env::set_current_dir(current_dir).map_err(TopiaryConfigFetchingError::Io)?;

        let path = match self.subdir.clone() {
            // Some grammars are in a subdirectory, go there
            Some(subdir) => tmp_dir.join(subdir),
            None => tmp_dir,
        }
        // parser.c and potential scanners are always in src/
        .join("src");

        self.build_tree_sitter_library(name, &path, library_path)
    }

    // NOTE: Much of the following code is heavily inspired by the `helix-loader` crate with license MPL-2.0.
    // To be safe, assume any and all of the following code is MLP-2.0 and copyrighted to the Helix project.
    fn build_tree_sitter_library(
        &self,
        name: &str,
        src_path: &PathBuf,
        target_path: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        log::info!("{}: compiling grammar", name);
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

        let mut config = cc::Build::new();
        config.cpp(true).opt_level(3).cargo_metadata(false);
        config.target(BUILD_TARGET);
        config.host(BUILD_TARGET);

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
            .arg("-I")
            .arg(header_path)
            .arg("-o")
            .arg(&target_path);

        if let Some(scanner_path) = scanner_path.as_ref() {
            if scanner_path.extension() == Some("c".as_ref()) {
                command.arg("-xc").arg("-std=c11").arg(scanner_path);
            } else {
                let mut cpp_command = Command::new(compiler.path());
                cpp_command.current_dir(src_path);
                for (key, value) in compiler.env() {
                    cpp_command.env(key, value);
                }
                cpp_command.args(compiler.args());
                let object_file = target_path.with_file_name(format!("{}_scanner.o", &self.rev));
                cpp_command
                    .arg("-fPIC")
                    .arg("-fno-exceptions")
                    .arg("-I")
                    .arg(header_path)
                    .arg("-o")
                    .arg(&object_file)
                    .arg("-std=c++14")
                    .arg("-c")
                    .arg(scanner_path);
                let output = cpp_command.output()?;
                if !output.status.success() {
                    return Err(TopiaryConfigFetchingError::Subprocess(format!(
                        "{:#?}, {:#?}",
                        output.stdout, output.stderr
                    )));
                }

                command.arg(&object_file);
            }
        }

        command.arg("-xc").arg("-std=c11").arg(parser_path);

        if cfg!(all(
            unix,
            not(any(target_os = "macos", target_os = "illumos"))
        )) {
            command.arg("-Wl,-z,relro,-z,now");
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(TopiaryConfigFetchingError::Subprocess(format!(
                "{:#?}, {:#?}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )));
        }

        log::info!("{}: successfully compiled", name);
        Ok(())
    }
}
