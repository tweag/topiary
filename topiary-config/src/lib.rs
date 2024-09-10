//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in Nickel, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.
pub mod error;
pub mod language;
pub mod source;

use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use language::{Language, LanguageConfiguration};
use nickel_lang_core::{eval::cache::CacheImpl, program::Program};
use serde::Deserialize;

use crate::{
    error::{TopiaryConfigError, TopiaryConfigResult},
    source::Source,
};

/// The configuration of the Topiary.
///
/// Contains information on how to format every language the user is interested in, modulo what is
/// supported. It can be provided by the user of the library, or alternatively, Topiary ships with
/// default configuration that can be accessed using `Configuration::default`.
#[derive(Debug)]
pub struct Configuration {
    languages: Vec<Language>,
}

/// Internal struct to help with deserialisation, converted to the actual Configuration in deserialization
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
struct SerdeConfiguration {
    languages: HashMap<String, LanguageConfiguration>,
}

impl Configuration {
    /// Consume the configuration from the usual sources.
    /// Which sources exactly can be read in the documentation of `Source`.
    ///
    /// # Errors
    ///
    /// If the configuration file does not exist, this function will return a `TopiaryConfigError`
    /// with the path that was not found.
    /// If the configuration file exists, but cannot be parsed, this function will return a
    /// `TopiaryConfigError` with the error that occurred.
    pub fn fetch(file: &Option<PathBuf>) -> TopiaryConfigResult<Self> {
        // If we have an explicit file, fail if it doesn't exist
        if let Some(path) = file {
            if !path.exists() {
                return Err(TopiaryConfigError::FileNotFound(path.to_path_buf()));
            }
        }

        // Otherwise, gather a list of all the files we want to look for
        let sources: Vec<Source> = Source::fetch(file);

        // And ask nickel to parse and merge them
        Self::parse_and_merge(&sources)
    }

    /// Gets a language configuration from the entire configuration.
    ///
    /// # Errors
    ///
    /// If the provided language name cannot be found in the `Configuration`, this
    /// function returns a `TopiaryConfigError`
    pub fn get_language<T>(&self, name: T) -> TopiaryConfigResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.languages
            .iter()
            .find(|language| language.name == name.as_ref())
            .ok_or(TopiaryConfigError::UnknownLanguage(name.to_string()))
    }

    /// Convenience alias to detect the Language from a Path-like value's extension.
    ///
    /// # Errors
    ///
    /// If the file extension is not supported, a `FormatterError` will be returned.
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> TopiaryConfigResult<&Language> {
        let pb = &path.as_ref().to_path_buf();
        if let Some(extension) = pb.extension().map(|ext| ext.to_string_lossy()) {
            for lang in &self.languages {
                if lang
                    .config
                    .extensions
                    .contains::<String>(&extension.to_string())
                {
                    return Ok(lang);
                }
            }
            return Err(TopiaryConfigError::UnknownExtension(extension.to_string()));
        }
        Err(TopiaryConfigError::NoExtension(pb.clone()))
    }

    fn parse_and_merge(sources: &[Source]) -> TopiaryConfigResult<Self> {
        let inputs = sources.iter().map(|s| s.clone().into());

        let mut program = Program::<CacheImpl>::new_from_inputs(inputs, std::io::stderr())?;

        let term = program.eval_full_for_export()?;

        let serde_config = SerdeConfiguration::deserialize(term)?;

        Ok(serde_config.into())
    }
}

impl Default for Configuration {
    /// Return the built-in configuration
    // This is particularly useful for testing
    fn default() -> Self {
        let mut program = Program::<CacheImpl>::new_from_source(
            Source::Builtin
                .read()
                .expect("Evaluating the builtin configuration should be safe")
                .as_slice(),
            "builtin",
            std::io::empty(),
        )
        .expect("Evaluating the builtin configuration should be safe");
        let term = program
            .eval_full_for_export()
            .expect("Evaluating the builtin configuration should be safe");
        let serde_config = SerdeConfiguration::deserialize(term)
            .expect("Evaluating the builtin configuration should be safe");

        serde_config.into()
    }
}

/// Convert `Serialisation` values into `HashMap`s, keyed on `Language::name`
impl From<&Configuration> for HashMap<String, Language> {
    fn from(config: &Configuration) -> Self {
        HashMap::from_iter(
            config
                .languages
                .iter()
                .map(|language| (language.name.clone(), language.clone())),
        )
    }
}

// Order-invariant equality; required for unit testing
impl PartialEq for Configuration {
    fn eq(&self, other: &Self) -> bool {
        let lhs: HashMap<String, Language> = self.into();
        let rhs: HashMap<String, Language> = other.into();

        lhs == rhs
    }
}

impl From<SerdeConfiguration> for Configuration {
    fn from(value: SerdeConfiguration) -> Self {
        let languages = value
            .languages
            .into_iter()
            .map(|(name, config)| Language::new(name, config))
            .collect();

        Self { languages }
    }
}

pub(crate) fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "", "topiary")
        .expect("Could not access the OS's Home directory")
}
