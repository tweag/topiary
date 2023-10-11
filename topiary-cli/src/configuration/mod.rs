//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in TOML, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.

mod collate;
mod fs;

use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fmt,
    path::PathBuf,
};

use clap::ValueEnum;
use indoc::formatdoc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::error::{CLIError, CLIResult, TopiaryError};

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, Deserialize, Serialize)]
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
pub struct Configuration {
    language: Vec<Language>,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration { language: vec![] }
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
    pub fn get_language<T: AsRef<str>>(&self, name: T) -> FormatterResult<&Language> {
        self.language
            .iter()
            .find(|&&language| language.name == name.as_ref())
            .ok_or(TopiaryError::Bin(
                format!("Unsupported language: \"{name}\""),
                Some(CLIError::UnsupportedLanguage(name.into())),
            ))
    }

    /// Default built-in languages.toml, parsed to a deserialised value.
    ///
    /// We do not parse to a `Configuration` value because the deserialsed TOML is easier to work
    /// with. Specifically, It allows additional configuration -- from other sources -- to be
    /// collated, to arrive at the final runtime configuration. (Parsing straight to
    /// `Configuration` doesn't work well, because that forces every configuration file to define
    /// every part of the configuration.)
    fn default_toml() -> toml::Value {
        let default_config = include_str!("../languages.toml");

        // We assume that the shipped built-in TOML is valid, so `.expect` is fine
        toml::from_str(default_config)
            .expect("Could not parse built-in languages.toml as valid TOML")
    }
}

/// Convert deserialised TOML values into `Configuration` values
impl TryFrom<toml::Value> for Configuration {
    type Error = TopiaryError;

    // This is particularly useful for testing
    fn try_from(toml: toml::Value) -> CLIResult<Self> {
        Configuration::default_toml()
            .try_into()
            .map_err(TopiaryError::from)
    }
}

/// Convert `Configuration` values into `HashMap`s, keyed on `Language::name`
// NOTE There are optimisations to be had here, to avoid cloning, but life's too short!
impl From<&Configuration> for HashMap<String, Language> {
    fn from(config: &Configuration) -> Self {
        HashMap::from_iter(config.language.iter().map(|language| {
            let name = language.name.clone();
            let language = language.clone();

            (name, language)
        }))
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

impl fmt::Display for Configuration {
    /// Pretty-print configuration as TOML
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string_pretty(self).map_err(|_| fmt::Error)?;
        write!(f, "{toml}")
    }
}

type Annotations = String;

/// Collation mode for configuration values
// NOTE The enum variants are in "natural" order, rather than
// sorted lexicographically, for the sake of the help text
#[derive(Clone, Debug, ValueEnum)]
pub enum CollationMode {
    /// When multiple sources of configuration are available, matching items are updated from the
    /// higher priority source, with collections merged as the union of sets.
    Merge,

    /// When multiple sources of configuration are available, matching items (including
    /// collections) are superseded from the higher priority source.
    Revise,

    /// When multiple sources of configuration are available, the highest priority source is taken.
    /// All values from lower priority sources are discarded.
    Override,
}

/// Map collation modes to merge depths for the TOML collation (see `collate_toml`)
impl From<&CollationMode> for usize {
    fn from(collation: &CollationMode) -> Self {
        match collation {
            CollationMode::Merge => 4,
            CollationMode::Revise => 2,
            _ => unreachable!(),
        }
    }
}

/// Consume the configuration from the usual sources, collated as specified
pub fn fetch(
    file: &Option<PathBuf>,
    collation: &CollationMode,
) -> CLIResult<(Annotations, Configuration)> {
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
            ConfigSource::Builtin => Ok(Configuration::default_toml()),

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
