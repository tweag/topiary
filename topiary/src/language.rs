use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Configuration, FormatterError, FormatterResult, IoError};

/// The languages that we support with query files.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Language {
    pub name: String,
    pub extensions: HashSet<String>,
    pub indent: Option<String>,
}

impl Language {
    /// Convenience alias to detect the Language from a Path-like value's extension.
    pub fn detect<P: AsRef<Path>>(path: P, config: &Configuration) -> FormatterResult<&Self> {
        let pb = path.as_ref().to_path_buf();
        if let Some(extension) = pb.extension().map(|ext| ext.to_string_lossy()) {
            for lang in &config.language {
                if lang.extensions.contains::<String>(&extension.to_string()) {
                    return Ok(lang);
                }
            }
        }
        todo!("ERIN: Error")
    }

    /// Convenience alias to return the query file path for the Language.
    pub fn query_file(&self) -> FormatterResult<PathBuf> {
        self.try_into()
    }

    /// Convert a Language into a vector of supported Tree-sitter grammars, ordered by priority.
    ///
    /// Note that, currently, all grammars are statically linked. This will change once dynamic linking
    /// is implemented (see Issue #4).
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn grammars(&self) -> FormatterResult<Vec<tree_sitter_facade::Language>> {
        Ok(match self.name.as_str() {
            "bash" => vec![tree_sitter_bash::language()],
            "json" => vec![tree_sitter_json::language()],
            "nickel" => vec![tree_sitter_nickel::language()],
            "ocaml" => vec![
                tree_sitter_ocaml::language_ocaml(),
                tree_sitter_ocaml::language_ocaml_interface(),
            ],
            "ocaml_implementation" => vec![tree_sitter_ocaml::language_ocaml()],
            "ocaml_interface" => vec![tree_sitter_ocaml::language_ocaml_interface()],
            "rust" => vec![tree_sitter_rust::language()],
            "toml" => vec![tree_sitter_toml::language()],
            "tree_sitter_query" => vec![tree_sitter_query::language()],
            name => todo!("ERIN: Error: {name}"),
        }
        .into_iter()
        .map(Into::into)
        .collect())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammars_wasm(&self) -> FormatterResult<Vec<tree_sitter_facade::Language>> {
        use futures::future::join_all;

        let language_names = match self.name.as_str() {
            "bash" => vec!["bash"],
            "json" => vec!["json"],
            "nickel" => vec!["nickel"],
            "ocaml" => vec!["ocaml", "ocaml_interface"],
            "ocamlImplementation" => vec!["ocaml"],
            "ocamlInterface" => vec!["ocaml_interface"],
            "rust" => vec!["rust"],
            "toml" => vec!["toml"],
            "treeSitterQuery" => vec!["query"],
        };

        Ok(join_all(language_names.iter().map(|name| async move {
            web_tree_sitter::Language::load_path(&format!(
                "/playground/scripts/tree-sitter-{}.wasm",
                name
            ))
            .await
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            let error: tree_sitter_facade::LanguageError = e.into();
            error
        })?
        .into_iter()
        .map(Into::into)
        .collect())
    }
}

/// Convert a Language into the canonical basename of its query file, under the most appropriate
/// search path. We test 3 different locations for query files, in the following priority order,
/// returning the first that exists:
///
/// 1. Under the TOPIARY_LANGUAGE_DIR environment variable at runtime;
/// 2. Under the TOPIARY_LANGUAGE_DIR environment variable at build time;
/// 3. Under the `./languages` subdirectory.
///
/// If all of these fail, we return an I/O error.
///
/// Note that different languages may map to the same query file, because their grammars produce
/// similar trees, which can be formatted with the same queries.
impl TryFrom<&Language> for PathBuf {
    type Error = FormatterError;

    fn try_from(language: &Language) -> FormatterResult<Self> {
        let basename = Self::from(match language.name.as_str() {
            "bash" => "bash",
            "json" => "json",
            "nickel" => "nickel",
            "ocaml" => "ocaml",
            "ocaml_interface" => "ocaml",
            "ocaml_implementation" => "ocaml",
            "rust" => "rust",
            "toml" => "toml",
            "tree_sitter_query" => "tree-sitter-query",
            name => todo!("ERIN: Error: {name}"),
        })
        .with_extension("scm");

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 4] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Some(PathBuf::from("./languages")),
            Some(PathBuf::from("../languages")),
        ];

        potentials
            .into_iter()
            .flatten()
            .map(|path| path.join(&basename))
            .find(|path| path.exists())
            .ok_or_else(|| {
                FormatterError::Io(IoError::Filesystem(
                    "Language query file could not be found".into(),
                    io::Error::from(io::ErrorKind::NotFound),
                ))
            })
    }
}
