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

                // Set the output path as the revision of the grammar,
                // with a platform-appropriate extension
                library_path.push(git_source.rev.clone());
                library_path.set_extension(std::env::consts::DLL_EXTENSION);

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

        self.fetch_and_compile_with_dir(name, library_path, false, tmp_dir.into_path())
    }

    pub fn fetch_and_compile_with_dir(
        &self,
        name: &str,
        library_path: PathBuf,
        force: bool,
        tmp_dir: PathBuf,
    ) -> Result<(), TopiaryConfigFetchingError> {
        if !force && library_path.is_file() {
            log::info!("{}: Built grammar already exists; nothing to do", name);
            return Ok(());
        }
        let tmp_dir = tmp_dir.join(name);

        // Clone the repository and checkout the configured revision
        log::info!("{}: Cloning from {}", name, self.git);
        Command::new("git")
            .arg("clone")
            .arg("--filter=blob:none")
            .arg(&self.git)
            .arg(&tmp_dir)
            .status()
            .map_err(TopiaryConfigFetchingError::Git)?;

        log::info!("{}: Checking out {}", name, self.rev);
        let current_dir = env::current_dir().map_err(TopiaryConfigFetchingError::Io)?;
        env::set_current_dir(&tmp_dir).map_err(TopiaryConfigFetchingError::Io)?;
        Command::new("git")
            .arg("checkout")
            .arg(&self.rev)
            .status()
            .map_err(TopiaryConfigFetchingError::Git)?;
        env::set_current_dir(current_dir).map_err(TopiaryConfigFetchingError::Io)?;

        // Update the build path for grammars that are not defined at the repo root
        let path = match self.subdir.clone() {
            // Some grammars are in a subdirectory, go there
            Some(subdir) => tmp_dir.join(subdir),
            None => tmp_dir,
        };

        // Build grammar
        log::info!("{name}: Building grammar");
        let mut loader =
            tree_sitter_loader::Loader::new().map_err(TopiaryConfigFetchingError::Build)?;
        loader.debug_build(false);
        loader.force_rebuild(true);
        loader
            .compile_parser_at_path(&path, library_path, &[])
            .map_err(TopiaryConfigFetchingError::Build)?;

        log::info!("{name}: Grammar successfully compiled");
        Ok(())
    }
}
