/// Topiary can be configured using the `Configuration` struct.
/// A basic configuration, written in toml, it is included buildtime and parsed runtime.
/// Additional configuration has to be provided by the user of the library.
use std::collections::HashSet;
use std::fmt;

use crate::{language::Language, FormatterError, FormatterResult};
use serde::{Deserialize, Serialize};

/// The configuration of Topiary. Contains information on how to format every language.
/// Can be provided by the user of the library, or alternatively, Topiary ships with a default
/// configuration that can be accessed using `default_configuration_toml` or
/// `parse_default_configuration`.
#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
    pub language: Vec<Language>,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration { language: vec![] }
    }

    /// Collects the known extensions of all languages into a single HashSet.
    /// Useful for testing if Topiary is able to configure the given file.
    #[must_use]
    pub fn known_extensions(&self) -> HashSet<&str> {
        let mut res: HashSet<&str> = HashSet::new();
        for lang in &self.language {
            for ext in &lang.extensions {
                res.insert(ext);
            }
        }
        res
    }

    /// Gets a language configuration from the entire configuration.
    ///
    /// # Errors
    ///
    /// If the provided language name cannot be found in the Configuration, this
    /// function returns a `FormatterError:UnsupportedLanguage`
    pub fn get_language<T: AsRef<str>>(&self, name: T) -> FormatterResult<&Language> {
        for lang in &self.language {
            if lang.name == name.as_ref() {
                return Ok(lang);
            }
        }
        return Err(FormatterError::UnsupportedLanguage(
            name.as_ref().to_string(),
        ));
    }

    /// Parse the default configuration directly into a `Configuration`,
    /// This is useful for users of Topiary that have no special requirements.
    /// It is also incredibly useful in tests.
    pub fn parse_default_configuration() -> FormatterResult<Self> {
        default_configuration_toml()
            .try_into()
            .map_err(FormatterError::from)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Configuration {
    /// Pretty-print configuration as TOML
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let toml = toml::to_string_pretty(self).map_err(|_| fmt::Error)?;
        write!(f, "{toml}")
    }
}

/// Default built-in languages.toml parsed to a toml file.
/// We parse the configuration file in two phases, the first is to a `toml::Value`
/// This function is exported to allow users of the library to merge their own
/// configuration with the builtin one.
/// Parsing straight to a `Configuration` doesn't work well, because that forces
/// every configuration file to define every part of the configuration.
pub fn default_configuration_toml() -> toml::Value {
    let default_config = include_str!("../languages.toml");
    toml::from_str(default_config).expect("Could not parse built-in languages.toml to valid toml")
}
