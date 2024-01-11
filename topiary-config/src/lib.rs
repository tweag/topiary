//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in TOML, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.

pub mod collate;
pub mod error;
pub mod serde;
mod source;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use indoc::formatdoc;
use itertools::Itertools;

use crate::{
    error::{TopiaryConfigError, TopiaryConfigResult},
    {collate::CollationMode, serde::Serialisation, source::Source},
};

use self::serde::Language;

/// Annotated configuration of the Topiary CLI
pub struct Configuration {
    annotations: String,
    configuration: Serialisation,
}

impl Configuration {
    /// Consume the configuration from the usual sources, collated as specified
    pub fn fetch(file: &Option<PathBuf>, collation: &CollationMode) -> TopiaryConfigResult<Self> {
        // If we have an explicit file, fail if it doesn't exist
        if let Some(path) = file {
            if !path.exists() {
                return Err(TopiaryConfigError::FileNotFound(path.to_path_buf()));
            }
        }

        let sources = Source::fetch(file);

        let annotations = annotate(&sources, collation);
        let configuration = configuration_toml(&sources, collation)?
            .try_into()
            .map_err(Into::<TopiaryConfigError>::into)?;

        Ok(Self {
            annotations,
            configuration,
        })
    }

    /// Gets a language configuration from the entire configuration.
    pub fn get_language<T>(&self, name: T) -> TopiaryConfigResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.configuration.get_language(name)
    }

    /// Convenience alias to detect the Language from a Path-like value's extension.
    ///
    /// # Errors
    ///
    /// If the file extension is not supported, a `FormatterError` will be returned.
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> TopiaryConfigResult<&Language> {
        self.configuration.detect(path)
    }
}

impl fmt::Display for Configuration {
    /// Pretty-print configuration as TOML, with annotations
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.annotations, self.configuration)
    }
}

impl Default for Configuration {
    /// Return the built-in configuration
    // This is particularly useful for testing
    // FIXME This *was* useful for testing, when it was part of the library. In the CLI, it may be
    // redundant...
    fn default() -> Self {
        // We assume that the built-in configuration is valid, so it's safe to unwrap
        Configuration::fetch(&None, &CollationMode::Merge).unwrap()
    }
}

/// Return annotations for the configuration in the form of TOML comments
/// (useful for human-readable output)
fn annotate(sources: &[Source], collation: &CollationMode) -> String {
    formatdoc!(
        "
        # Configuration collated from the following sources,
        # in priority order (lowest to highest):
        #
        {}
        #
        # Collation mode: {collation:?}
        ",
        sources
            .iter()
            .enumerate()
            .map(|(i, source)| format!("# {}. {source}", i + 1))
            .join("\n")
    )
}

/// Consume configuration and collate as specified
fn configuration_toml(
    sources: &[Source],
    collation: &CollationMode,
) -> TopiaryConfigResult<toml::Value> {
    match collation {
        CollationMode::Override => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .last()
                .unwrap()
                .try_into()
                .map_err(Into::<TopiaryConfigError>::into)
        }

        // CollationMode::Merge and CollationMode::Revise
        _ => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .iter()
                .map(|source| source.try_into())
                .reduce(|config, toml| Ok(collation.collate_toml(config?, toml?)))
                .unwrap()
        }
    }
}
