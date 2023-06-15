use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
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
    ///
    /// # Errors
    ///
    /// If the file extension is not supported, a `FormatterError` will be returned.
    pub fn detect<P: AsRef<Path>>(path: P, config: &Configuration) -> FormatterResult<&Self> {
        let pb = &path.as_ref().to_path_buf();
        if let Some(extension) = pb.extension().map(|ext| ext.to_string_lossy()) {
            for lang in &config.language {
                if lang.extensions.contains::<String>(&extension.to_string()) {
                    return Ok(lang);
                }
            }
            return Err(FormatterError::LanguageDetection(
                pb.clone(),
                Some(extension.to_string()),
            ));
        }
        Err(FormatterError::LanguageDetection(pb.clone(), None))
    }

    /// Convenience alias to return the query file path for the Language.
    pub fn query_file(&self) -> FormatterResult<PathBuf> {
        self.try_into()
    }

    /// Convert a Language into a vector of supported Tree-sitter grammars, ordered by priority.
    ///
    /// Note that, currently, all grammars are statically linked. This will change once dynamic linking
    /// is implemented (see Issue #4).
    ///
    /// # Errors
    ///
    /// If the language is not supported, a `FormatterError` will be returned.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn grammar(&self) -> FormatterResult<tree_sitter_facade::Language> {
        Ok(match self.name.as_str() {
            "bash" => tree_sitter_bash::language(),
            "json" => tree_sitter_json::language(),
            "nickel" => tree_sitter_nickel::language(),
            "ocaml" => tree_sitter_ocaml::language_ocaml(),
            "ocaml_interface" => tree_sitter_ocaml::language_ocaml_interface(),
            "rust" => tree_sitter_rust::language(),
            "toml" => tree_sitter_toml::language(),
            "tree_sitter_query" => tree_sitter_query::language(),
            name => return Err(FormatterError::UnsupportedLanguage(name.to_string())),
        }
        .into())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammar_wasm(&self) -> FormatterResult<tree_sitter_facade::Language> {
        let language_name = match self.name.as_str() {
            "bash" => "bash",
            "json" => "json",
            "nickel" => "nickel",
            "ocaml" => "ocaml",
            "ocaml_interface" => "ocaml_interface",
            "rust" => "rust",
            "toml" => "toml",
            "tree_sitter_query" => "query",
            name => return Err(FormatterError::UnsupportedLanguage(name.to_string())),
        };

        Ok(web_tree_sitter::Language::load_path(&format!(
            "/playground/scripts/tree-sitter-{language_name}.wasm"
        ))
        .await
        .map_err(|e| {
            let error: tree_sitter_facade::LanguageError = e.into();
            error
        })?
        .into())
    }
}

/// Convert a Language into the canonical basename of its query file, under the most appropriate
/// search path. We test 3 different locations for query files, in the following priority order,
/// returning the first that exists:
///
/// 1. Under the `TOPIARY_LANGUAGE_DIR` environment variable at runtime;
/// 2. Under the `TOPIARY_LANGUAGE_DIR` environment variable at build time;
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
            "ocaml" | "ocaml_interface" => "ocaml",
            "rust" => "rust",
            "toml" => "toml",
            "tree_sitter_query" => "tree-sitter-query",
            name => return Err(FormatterError::UnsupportedLanguage(name.to_string())),
        })
        .with_extension("scm");

        #[rustfmt::skip]
        let potentials: [Option<Self>; 4] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(Self::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(Self::from),
            Some(Self::from("./languages")),
            Some(Self::from("../languages")),
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

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SupportedLanguage {
    Json,
    Nickel,
    Ocaml,
    OcamlInterface,
    Toml,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

impl SupportedLanguage {
    pub fn to_language<'config>(&self, configuration: &'config Configuration) -> &'config Language {
        let name = self.name();

        for lang in &configuration.language {
            if lang.name == name {
                return lang;
            }
        }

        // Every supported language MUST have an entry in the builtin
        // configuration, and so there should always be a match.
        unreachable!()
    }

    pub fn name(&self) -> &str {
        match self {
            SupportedLanguage::Json => "json",
            SupportedLanguage::Nickel => "nickel",
            SupportedLanguage::Ocaml => "ocaml",
            SupportedLanguage::OcamlInterface => "ocaml_interface",
            SupportedLanguage::Toml => "toml",
        }
    }

    pub fn is_supported(name: &str) -> bool {
        SupportedLanguage::from_str(name, true).is_ok()
    }
}
