//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in TOML, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.

mod collate;
pub mod format;
mod source;

use std::path::PathBuf;

use indoc::formatdoc;
use itertools::Itertools;

use crate::error::{CLIResult, TopiaryError};
pub use collate::CollationMode;
use source::ConfigSource;

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

    let sources = ConfigSource::fetch(file);

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
                .reduce(|config, toml| Ok(collation.collate_toml(config?, toml?)))
                .unwrap()
        }
    }
}
