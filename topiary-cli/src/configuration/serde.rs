//! Configuration serialisation and deserialisation

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::error::{CLIError, CLIResult, TopiaryError};

// TODO Should `Language` be in crate::language?...

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in `Configuration` and
    /// to convert to the respective Tree-sitter grammar
    pub name: String,

    /// A set of the filetype extensions associated with this language. This enables Topiary to
    /// switch to the right language based on the input filename.
    pub extensions: HashSet<String>,

    /// The indentation string used for this language; defaults to "  " (i.e., two spaces). Any
    /// string can be provided, but in most instances it will be some whitespace (e.g., "    ",
    /// "\t", etc.)
    indent: Option<String>,
}

// TODO I don't think we're going to need this here...but maybe
impl Language {
    pub fn indent(&self) -> &str {
        match self.indent {
            Some(indent) => &indent,
            None => "  ",
        }
    }
}

/// The configuration of the Topiary CLI.
///
/// Contains information on how to format every language the user is interested in, modulo what is
/// supported. It can be provided by the user of the library, or alternatively, Topiary ships with
/// default configuration that can be accessed using `Configuration::default_toml`.
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
    /// If the provided language name cannot be found in the `Configuration`, this
    /// function returns a `TopiaryError`
    pub fn get_language<T>(&self, name: T) -> CLIResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.language
            .iter()
            .find(|&&language| language.name == name.as_ref())
            .ok_or(TopiaryError::Bin(
                format!("Unsupported language: \"{name}\""),
                Some(CLIError::UnsupportedLanguage(name.to_string())),
            ))
    }

    /// Default built-in languages.toml, parsed to a deserialised value.
    ///
    /// We do not parse to a `Configuration` value because the deserialsed TOML is easier to work
    /// with. Specifically, It allows additional configuration -- from other sources -- to be
    /// collated, to arrive at the final runtime configuration. (Parsing straight to
    /// `Configuration` doesn't work well, because that forces every configuration file to define
    /// every part of the configuration.)
    pub fn default_toml() -> toml::Value {
        let default_config = include_str!("../../../languages.toml");

        // We assume that the shipped built-in TOML is valid, so `.expect` is fine
        toml::from_str(default_config)
            .expect("Could not parse built-in languages.toml as valid TOML")
    }
}

/// Convert deserialised TOML values into `Configuration` values
impl TryFrom<toml::Value> for Serialisation {
    type Error = TopiaryError;

    // This is particularly useful for testing
    fn try_from(toml: toml::Value) -> CLIResult<Self> {
        Serialisation::default_toml()
            .try_into()
            .map_err(TopiaryError::from)
    }
}

/// Convert `Configuration` values into `HashMap`s, keyed on `Language::name`
impl From<&Serialisation> for HashMap<String, Language> {
    fn from(config: &Serialisation) -> Self {
        HashMap::from_iter(
            config
                .language
                .iter()
                .map(|language| (language.name, *language)),
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
