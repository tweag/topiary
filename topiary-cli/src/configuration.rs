//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in TOML, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.

mod collate;
pub mod format;
mod fs;

use std::{fmt, path::PathBuf};

use indoc::formatdoc;
use itertools::Itertools;

use crate::error::{CLIResult, TopiaryError};
use collate::collate_toml;
pub use collate::CollationMode;
use fs::{find_os_configuration_dir, find_workspace_configuration_dir};

type Annotations = String;

/// Consume the configuration from the usual sources, collated as specified
pub fn fetch(
    file: &Option<PathBuf>,
    collation: &CollationMode,
) -> CLIResult<(Annotations, format::Configuration)> {
    // If we have an explicit file, fail if it doesn't exist
    if let Some(path) = file {
        if !path.exists() {
            return Err(TopiaryError::Bin(
                format!("Configuration file not found: {}", path.to_string_lossy()),
                None,
            ));
        }
    }

    let sources = configuration_sources(file);

    Ok((
        annotate(&sources, collation),
        configuration_toml(&sources, collation)?
            .try_into()
            .map_err(TopiaryError::from)?,
    ))
}

/// Return annotations for the configuration in the form of TOML comments
/// (useful for human-readable output)
fn annotate(sources: &[ConfigSource], collation: &CollationMode) -> String {
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

/// Sources of TOML configuration
#[derive(Debug)]
enum ConfigSource {
    Builtin,
    File(PathBuf),

    // This is a sentinel element for files that don't exist
    Missing,
}

impl ConfigSource {
    fn is_valid(&self) -> bool {
        !matches!(self, Self::Missing)
    }
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Builtin => write!(f, "Built-in configuration"),

            Self::File(path) => {
                // We only stringify the path when we know it exists, so the call to `canonicalize`
                // is safe to unwrap. (All bets are off, if called from elsewhere.)
                write!(f, "{}", path.canonicalize().unwrap().to_string_lossy())
            }

            Self::Missing => write!(f, "Missing configuration"),
        }
    }
}

impl From<Option<PathBuf>> for ConfigSource {
    fn from(path: Option<PathBuf>) -> Self {
        match path {
            None => ConfigSource::Missing,

            Some(path) => {
                let candidate = if path.is_dir() {
                    path.join("languages.toml")
                } else {
                    path
                };

                if candidate.exists() {
                    ConfigSource::File(candidate)
                } else {
                    log::warn!(
                        "Could not find configuration file: {}",
                        candidate.to_string_lossy()
                    );

                    ConfigSource::Missing
                }
            }
        }
    }
}

impl TryFrom<&ConfigSource> for toml::Value {
    type Error = TopiaryError;

    fn try_from(source: &ConfigSource) -> Result<Self, Self::Error> {
        match source {
            ConfigSource::Builtin => Ok(format::Configuration::default_toml()),

            ConfigSource::File(file) => {
                let config = std::fs::read_to_string(file)?;
                toml::from_str(&config).map_err(TopiaryError::from)
            }

            ConfigSource::Missing => Err(TopiaryError::Bin(
                "Could not parse missing configuration".into(),
                None,
            )),
        }
    }
}

/// Return the valid sources of configuration, in priority order (lowest to highest):
///
/// 1. Built-in configuration (per `Configuration::default_toml()`)
/// 2. `~/.config/topiary/languages.toml` (or equivalent)
/// 3. `.topiary/languages.toml` (or equivalent)
/// 4. `file`, passed as a CLI argument/environment variable
fn configuration_sources(file: &Option<PathBuf>) -> Vec<ConfigSource> {
    [
        ConfigSource::Builtin,
        Some(find_os_configuration_dir()).into(),
        find_workspace_configuration_dir().into(),
        file.clone().into(),
    ]
    .into_iter()
    .filter(ConfigSource::is_valid)
    .collect()
}

/// Consume configuration and collate as specified
fn configuration_toml(
    sources: &[ConfigSource],
    collation: &CollationMode,
) -> CLIResult<toml::Value> {
    match collation {
        CollationMode::Override => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .last()
                .unwrap()
                .try_into()
                .map_err(TopiaryError::from)
        }

        // CollationMode::Merge and CollationMode::Revise
        _ => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .iter()
                .map(|source| source.try_into())
                .reduce(|config, toml| Ok(collate_toml(config?, toml?, collation)))
                .unwrap()
        }
    }
}
