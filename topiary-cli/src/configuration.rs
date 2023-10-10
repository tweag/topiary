//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in TOML, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.

use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fmt,
    path::PathBuf,
};

use clap::ValueEnum;
use directories::ProjectDirs;
use indoc::formatdoc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::error::{CLIResult, TopiaryError};

// TODO
use crate::{language::Language, FormatterError, FormatterResult};

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
            ConfigSource::Builtin => Ok(default_configuration_toml()),

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
/// 1. Built-in configuration (`topiary::default_configuration_toml`)
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

/// Find the OS-specific configuration directory
fn find_os_configuration_dir() -> PathBuf {
    ProjectDirs::from("", "", "topiary")
        .expect("Could not access the OS's Home directory")
        .config_dir()
        .to_path_buf()
}

/// Ascend the directory hierarchy, starting from the current working directory, in search of the
/// nearest `.topiary` configuration directory
fn find_workspace_configuration_dir() -> Option<PathBuf> {
    current_dir()
        .expect("Could not get current working directory")
        .ancestors()
        .map(|path| path.join(".topiary"))
        .find(|path| path.exists())
}

/// Collate two TOML documents, merging values from `graft` onto `base`.
///
/// Arrays of tables with a `name` key (e.g., our `[[language]]` tables) are always merged; that
/// is, the union of the `base` and `graft` is taken. Otherwise, the `merge_depth` controls the
/// collation of arrays, resulting in concatenation. This can leave duplicates, in the collated
/// TOML, but for Topiary, this only matters for our `Languages::extensions`, which is implemented
/// as a `HashSet`; thus deserialisation will deduplicate for us.
///
/// When a table exists in both `base` and `graft`, the merged table consists of all keys in
/// `base`'s table unioned with all keys in `graft` with the values of `graft` being merged
/// recursively onto values of `base`.
///
/// NOTE This collation function is forked from Helix, licensed under MPL-2.0
/// * Repo: https://github.com/helix-editor/helix
/// * Rev:  df09490
/// * Path: helix-loader/src/lib.rs
fn collate_toml<T>(base: toml::Value, graft: toml::Value, merge_depth: T) -> toml::Value
where
    T: Into<usize>,
{
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    let merge_depth: usize = merge_depth.into();

    match (base, graft, merge_depth) {
        // Fallback to the graft value if the recursion depth bottoms out
        (_, graft, 0) => graft,

        (Value::Array(mut base_items), Value::Array(graft_items), _) => {
            for rvalue in graft_items {
                // If our graft value has a `name` key, then we're dealing with a `[[language]]`
                // table. In which case, pop it -- if it exists -- from the base array.
                let language = get_name(&rvalue)
                    .and_then(|rname| base_items.iter().position(|v| get_name(v) == Some(rname)))
                    .map(|lpos| base_items.remove(lpos));

                let mvalue = match language {
                    // Merge matching language tables
                    Some(lvalue) => collate_toml(lvalue, rvalue, merge_depth - 1),

                    // Collate everything else
                    None => rvalue,
                };

                base_items.push(mvalue);
            }

            Value::Array(base_items)
        }

        (Value::Table(mut base_map), Value::Table(graft_map), _) => {
            for (rname, rvalue) in graft_map {
                match base_map.remove(&rname) {
                    Some(lvalue) => {
                        let merged_value = collate_toml(lvalue, rvalue, merge_depth - 1);
                        base_map.insert(rname, merged_value);
                    }
                    None => {
                        base_map.insert(rname, rvalue);
                    }
                }
            }

            Value::Table(base_map)
        }

        // Fallback to the graft value for everything else
        (_, graft, _) => graft,
    }
}

#[cfg(test)]
mod test_config_collation {
    use super::{collate_toml, CollationMode, Configuration};

    // NOTE PartialEq for toml::Value is (understandably) order sensitive over array elements, so
    // we deserialse to `topiary::Configuration` for equality testing. This also has the effect of
    // side-stepping potential duplication, from concatenation, when using `CollationMode::Merge`.

    static BASE: &str = r#"
        [[language]]
        name = "example"
        extensions = ["eg"]

        [[language]]
        name = "demo"
        extensions = ["demo"]
    "#;

    static GRAFT: &str = r#"
        [[language]]
        name = "example"
        extensions = ["example"]
        indent = "\t"
    "#;

    #[test]
    fn merge() {
        let base = toml::from_str(BASE).unwrap();
        let graft = toml::from_str(GRAFT).unwrap();

        let merged: Configuration = collate_toml(base, graft, &CollationMode::Merge)
            .try_into()
            .unwrap();

        let expected: Configuration = toml::from_str(
            r#"
            [[language]]
            name = "example"
            extensions = ["eg", "example"]
            indent = "\t"

            [[language]]
            name = "demo"
            extensions = ["demo"]
            "#,
        )
        .unwrap();

        assert_eq!(merged, expected);
    }

    #[test]
    fn revise() {
        let base = toml::from_str(BASE).unwrap();
        let graft = toml::from_str(GRAFT).unwrap();

        let revised: Configuration = collate_toml(base, graft, &CollationMode::Revise)
            .try_into()
            .unwrap();

        let expected: Configuration = toml::from_str(
            r#"
            [[language]]
            name = "example"
            extensions = ["example"]
            indent = "\t"

            [[language]]
            name = "demo"
            extensions = ["demo"]
            "#,
        )
        .unwrap();

        assert_eq!(revised, expected);
    }
}
