use std::io;
use std::path::{Path, PathBuf};

use crate::{FormatterError, FormatterResult, IoError};

/// The languages that we support with query files.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Bash,
    Json,
    Nickel,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    Rust,
    Toml,
    TreeSitterQuery,
}

// NOTE This list of extension mappings is influenced by Wilfred Hughes' Difftastic
// https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
const EXTENSIONS: &[(Language, &[&str])] = &[
    (Language::Bash, &["sh", "bash"]),
    (
        Language::Json,
        &[
            "json",
            "avsc",
            "geojson",
            "gltf",
            "har",
            "ice",
            "JSON-tmLanguage",
            "jsonl",
            "mcmeta",
            "tfstate",
            "tfstate.backup",
            "topojson",
            "webapp",
            "webmanifest",
        ],
    ),
    (Language::Nickel, &["ncl"]),
    (Language::OcamlImplementation, &["ml"]),
    (Language::OcamlInterface, &["mli"]),
    (Language::Rust, &["rs"]),
    (Language::Toml, &["toml"]),
    (Language::TreeSitterQuery, &["scm"]),
];

impl Language {
    /// Convenience alias to create a Language from "magic strings".
    pub fn new(s: &str) -> FormatterResult<Self> {
        s.try_into()
    }

    /// Convenience alias to detect the Language from a Path-like value's extension.
    pub fn detect<P: AsRef<Path>>(path: P) -> FormatterResult<Self> {
        path.as_ref().to_path_buf().try_into()
    }

    /// Convenience alias to return the query file path for the Language.
    pub fn query_file(&self) -> FormatterResult<PathBuf> {
        self.try_into()
    }

    /// Convenience alias to return the Tree-sitter grammars for the Language.
    pub fn grammars(&self) -> Vec<tree_sitter_facade::Language> {
        self.into()
    }
}

/// Convert a string into a Language, if possible.
impl TryFrom<&str> for Language {
    type Error = FormatterError;

    fn try_from(s: &str) -> FormatterResult<Self> {
        Ok(match s.to_lowercase().as_str() {
            "bash" => Language::Bash,
            "json" => Language::Json,
            "nickel" => Language::Nickel,
            "ocaml" => Language::Ocaml,
            "ocaml-implementation" => Language::OcamlImplementation,
            "ocaml-interface" => Language::OcamlInterface,
            "rust" => Language::Rust,
            "toml" => Language::Toml,
            "tree-sitter-query" => Language::TreeSitterQuery,

            _ => {
                return Err(FormatterError::Query(
                    format!("Unsupported language specified: '{s}'"),
                    None,
                ))
            }
        })
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
        let basename = Self::from(match language {
            Language::Bash => "bash",
            Language::Json => "json",
            Language::Nickel => "nickel",
            Language::Ocaml => "ocaml",
            Language::OcamlImplementation => "ocaml",
            Language::OcamlInterface => "ocaml",
            Language::Rust => "rust",
            Language::Toml => "toml",
            Language::TreeSitterQuery => "tree-sitter-query",
        })
        .with_extension("scm");

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 3] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Some(PathBuf::from("./languages")),
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

/// Extract the extension from a Path and use it to detect the Language.
///
/// Note that, ideally, we'd like to TryFrom AsRef<Path>, but this collides with a blanket
/// implementation in core :(
impl TryFrom<PathBuf> for Language {
    type Error = FormatterError;

    fn try_from(path: PathBuf) -> FormatterResult<Self> {
        let extension = path.extension().map(|ext| ext.to_string_lossy());

        if let Some(extension) = &extension {
            // NOTE This extension search is influenced by Wilfred Hughes' Difftastic
            // https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
            for (language, extensions) in EXTENSIONS {
                if extensions.iter().any(|&candidate| candidate == extension) {
                    return Ok(*language);
                }
            }
        }

        Err(FormatterError::LanguageDetection(
            path.clone(),
            extension.map(|v| v.into()),
        ))
    }
}

/// Convert a Language into a vector of supported Tree-sitter grammars, ordered by priority.
///
/// Note that, currently, all grammars are statically linked. This will change once dynamic linking
/// is implemented (see Issue #4).
#[cfg(not(target_arch = "wasm32"))]
impl From<&Language> for Vec<tree_sitter_facade::Language> {
    fn from(language: &Language) -> Self {
        match language {
            Language::Bash => vec![tree_sitter_bash::language()],
            Language::Json => vec![tree_sitter_json::language()],
            Language::Nickel => vec![tree_sitter_nickel::language()],
            Language::Ocaml => vec![
                tree_sitter_ocaml::language_ocaml(),
                tree_sitter_ocaml::language_ocaml_interface(),
            ],
            Language::OcamlImplementation => vec![tree_sitter_ocaml::language_ocaml()],
            Language::OcamlInterface => vec![tree_sitter_ocaml::language_ocaml_interface()],
            Language::Rust => vec![tree_sitter_rust::language()],
            Language::Toml => vec![tree_sitter_toml::language()],
            Language::TreeSitterQuery => vec![tree_sitter_query::language()],
        }
        .into_iter()
        .map(Into::into)
        .collect()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<&Language> for Vec<tree_sitter_facade::Language> {
    fn from(language: &Language) -> Self {
        match language {
            Language::Bash => vec!["bash"],
            Language::Json => vec!["json"],
            Language::Nickel => vec!["nickel"],
            Language::Ocaml => vec!["ocaml", "ocaml-interface"],
            Language::OcamlImplementation => vec!["ocaml"],
            Language::OcamlInterface => vec!["ocaml-interface"],
            Language::Rust => vec!["rust"],
            Language::Toml => vec!["toml"],
            Language::TreeSitterQuery => vec!["query"],
        }
        .iter()
        .map(|name| format!("tree-sitter-{}.wasm", name))
        .map(|path| {
            web_tree_sitter::Language::load_path(path)
                .await
                .map_err(|e| {
                    let error: web_tree_sitter::Language::LanguageError = e.into();
                    error
                })?
                .into();
        })
    }
}
