//! Configuration serialisation and deserialisation

use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::{TopiaryConfigError, TopiaryConfigResult};

// TODO Should `Language` be in crate::language?...

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in the deserialised
    /// configuration and to convert to the respective Tree-sitter grammar
    pub name: String,

    /// A set of the filetype extensions associated with this language. This enables Topiary to
    /// switch to the right language based on the input filename.
    pub extensions: HashSet<String>,

    /// The indentation string used for this language; defaults to "  " (i.e., two spaces). Any
    /// string can be provided, but in most instances it will be some whitespace (e.g., "    ",
    /// "\t", etc.)
    pub indent: Option<String>,
}

// TODO I don't think we're going to need this here...but maybe
impl Language {
    #[cfg(not(wasm))]
    pub fn find_query_file(&self) -> TopiaryConfigResult<PathBuf> {
        let basename = PathBuf::from(match self.name.as_str() {
            "bash" => "bash",
            "json" => "json",
            "nickel" => "nickel",
            "ocaml" | "ocaml_interface" => "ocaml",
            "ocamllex" => "ocamllex",
            "rust" => "rust",
            "toml" => "toml",
            "tree_sitter_query" => "tree-sitter-query",
            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        })
        .with_extension("scm");

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
    pub fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
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
            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        }
        .into())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
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
            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        };

        Ok(topiary_web_tree_sitter_sys::Language::load_path(&format!(
            "/playground/scripts/tree-sitter-{language_name}.wasm"
        ))
        .await
        .map_err(|e| {
            let error: topiary_tree_sitter_facade::LanguageError = e.into();
            error
        })?
        .into())
    }
}

/// The configuration of the Topiary.
///
/// Contains information on how to format every language the user is interested in, modulo what is
/// supported. It can be provided by the user of the library, or alternatively, Topiary ships with
/// default configuration that can be accessed using `Serialisation::default_toml`.
#[derive(Deserialize, Serialize, Debug)]
pub struct Serialisation {
    language: Vec<Language>,
}

impl Serialisation {
    pub fn new() -> Self {
        Serialisation { language: vec![] }
    }

    /// Collects the known extensions of all languages into a single HashSet.
    /// Useful for testing if Topiary is able to configure the given file.
    pub fn known_extensions(&self) -> HashSet<String> {
        self.language
            .iter()
            .fold(HashSet::new(), |extensions, language| {
                &extensions | &language.extensions
            })
    }

    /// Gets a language configuration from the entire configuration.
    ///
    /// # Errors
    ///
    /// If the provided language name cannot be found in the `Serialisation`, this
    /// function returns a `TopiaryError`
    pub fn get_language<T>(&self, name: T) -> TopiaryConfigResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.language
            .iter()
            .find(|language| language.name == name.as_ref())
            .ok_or(TopiaryConfigError::UnknownLanguage(name.to_string()))
    }

    /// Default built-in languages.toml, parsed to a deserialised value.
    ///
    /// We do not parse to a `Serialisation` value because the deserialsed TOML is easier to work
    /// with. Specifically, It allows additional configuration -- from other sources -- to be
    /// collated, to arrive at the final runtime configuration. (Parsing straight to
    /// `Serialisation` doesn't work well, because that forces every configuration file to define
    /// every part of the configuration.)
    pub fn default_toml() -> toml::Value {
        let default_config = include_str!("../languages.toml");

        // We assume that the shipped built-in TOML is valid, so `.expect` is fine
        toml::from_str(default_config)
            .expect("Could not parse built-in languages.toml as valid TOML")
    }

    /// Convenience alias to detect the Language from a Path-like value's extension.
    ///
    /// # Errors
    ///
    /// If the file extension is not supported, a `FormatterError` will be returned.
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> TopiaryConfigResult<&Language> {
        let pb = &path.as_ref().to_path_buf();
        if let Some(extension) = pb.extension().map(|ext| ext.to_string_lossy()) {
            for lang in &self.language {
                if lang.extensions.contains::<String>(&extension.to_string()) {
                    return Ok(lang);
                }
            }
            return Err(TopiaryConfigError::UnknownExtension(extension.to_string()));
        }
        Err(TopiaryConfigError::NoExtension(pb.clone()))
    }
}

impl Default for Serialisation {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert deserialised TOML values into `Serialisation` values
// TODO Is this necessary, any more?
impl TryFrom<toml::Value> for Serialisation {
    type Error = TopiaryConfigError;

    fn try_from(toml: toml::Value) -> TopiaryConfigResult<Self> {
        toml.try_into().map_err(TopiaryConfigError::from)
    }
}

/// Convert `Serialisation` values into `HashMap`s, keyed on `Language::name`
impl From<&Serialisation> for HashMap<String, Language> {
    fn from(config: &Serialisation) -> Self {
        HashMap::from_iter(
            config
                .language
                .iter()
                .map(|language| (language.name.clone(), language.clone())),
        )
    }
}

// Order-invariant equality; required for unit testing
impl PartialEq for Serialisation {
    fn eq(&self, other: &Self) -> bool {
        let lhs: HashMap<String, Language> = self.into();
        let rhs: HashMap<String, Language> = other.into();

        lhs == rhs
    }
}

impl fmt::Display for Serialisation {
    /// Pretty-print configuration as TOML
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string_pretty(self).map_err(|_| fmt::Error)?;
        write!(f, "{toml}")
    }
}
