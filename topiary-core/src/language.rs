use std::{
    collections::HashSet,
    fmt, io,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{Configuration, FormatterError, FormatterResult, IoError};

/// A Language contains all the information Topiary requires to format that
/// specific languages.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in
    /// the Configuration, and to convert from a language to the respective tree-sitter
    /// grammar.
    pub name: String,
    /// A Set of the filetype extensions associated with this particular language.
    /// Enables Topiary to pick the right language given an input file
    pub extensions: HashSet<String>,
    /// The indentation string used for that particular language. Defaults to "  "
    /// if not provided. Any string can be provided, but in most instances will be
    /// some whitespace: "  ", "    ", or "\t".
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

    /// Convert a Language into a supported Tree-sitter grammar.
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
            "ocamllex" => tree_sitter_ocamllex::language(),
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
            "ocamllex" => "ocamllex",
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

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Convert a Language into the canonical basename of its query file, under the most appropriate
/// search path. We test 3 different locations for query files, in the following priority order,
/// returning the first that exists:
///
/// 1. Under the `TOPIARY_LANGUAGE_DIR` environment variable at runtime;
/// 2. Under the `TOPIARY_LANGUAGE_DIR` environment variable at build time;
/// 3. Under the `./queries` subdirectory.
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
            "ocamllex" => "ocamllex",
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
            Some(Self::from("./topiary-queries/queries")),
            Some(Self::from("../topiary-queries/queries")),
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

/// Topiary can format more languages than are actually "supported".
/// This enum is an enumeration of those we (the maintainers) are comfortable in
/// calling "supported".
/// Any other entries in crate::Language are experimental and won't be
/// exposed in the CLI. They can be accessed using --query language/foo.scm
/// instead.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SupportedLanguage {
    Json,
    Nickel,
    Ocaml,
    OcamlInterface,
    Ocamllex,
    Toml,
}

impl SupportedLanguage {
    /// Function to convert a `SupportedLanguage` into a `crate::Language` for further processing
    pub fn to_language<'config>(&self, configuration: &'config Configuration) -> &'config Language {
        let name = self.name();

        for lang in &configuration.language {
            if lang.name == name {
                return lang;
            }
        }

        unreachable!("A match should always be returned because every supported language must have an entry in the builtin configuration file")
    }

    pub fn name(&self) -> &str {
        match self {
            SupportedLanguage::Json => "json",
            SupportedLanguage::Nickel => "nickel",
            SupportedLanguage::Ocaml => "ocaml",
            SupportedLanguage::OcamlInterface => "ocaml_interface",
            SupportedLanguage::Ocamllex => "ocamllex",
            SupportedLanguage::Toml => "toml",
        }
    }

    pub fn is_supported(name: &str) -> bool {
        SupportedLanguage::from_str(name, true).is_ok()
    }
}
