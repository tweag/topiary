use clap::ValueEnum;
use directories::ProjectDirs;
use indoc::formatdoc;
use itertools::Itertools;
use std::{env::current_dir, fmt, path::PathBuf};
use topiary::{default_configuration_toml, Configuration};

use crate::error::{CLIResult, TopiaryError};

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
        CollationMode::Merge => todo!(),

        CollationMode::Revise => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .iter()
                .map(|source| source.try_into())
                .reduce(|config, toml| Ok(merge_toml_values(config?, toml?, 3)))
                .unwrap()
        }

        CollationMode::Override => {
            // It's safe to unwrap here, as `sources` is guaranteed to contain at least one element
            sources
                .last()
                .unwrap()
                .try_into()
                .map_err(TopiaryError::from)
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

/// Merge two TOML documents, merging values from `right` onto `left`
///
/// When an array exists in both `left` and `right`, `right`'s array is
/// used. When a table exists in both `left` and `right`, the merged table
/// consists of all keys in `left`'s table unioned with all keys in `right`
/// with the values of `right` being merged recursively onto values of
/// `left`.
///
/// `merge_toplevel_arrays` controls whether a top-level array in the TOML
/// document is merged instead of overridden. This is useful for TOML
/// documents that use a top-level array of values like the `languages.toml`,
/// where one usually wants to override or add to the array instead of
/// replacing it altogether.
///
/// NOTE: This merge function is taken from Helix:
/// https://github.com/helix-editor/helix licensed under MPL-2.0. There
/// it is defined under: helix-loader/src/lib.rs. Taken from commit df09490
pub fn merge_toml_values(left: toml::Value, right: toml::Value, merge_depth: usize) -> toml::Value {
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    match (left, right) {
        (Value::Array(mut left_items), Value::Array(right_items)) => {
            // The top-level arrays should be merged but nested arrays should
            // act as overrides. For the `languages.toml` config, this means
            // that you can specify a sub-set of languages in an overriding
            // `languages.toml` but that nested arrays like file extensions
            // arguments are replaced instead of merged.
            if merge_depth > 0 {
                left_items.reserve(right_items.len());
                for rvalue in right_items {
                    let lvalue = get_name(&rvalue)
                        .and_then(|rname| {
                            left_items.iter().position(|v| get_name(v) == Some(rname))
                        })
                        .map(|lpos| left_items.remove(lpos));
                    let mvalue = match lvalue {
                        Some(lvalue) => merge_toml_values(lvalue, rvalue, merge_depth - 1),
                        None => rvalue,
                    };
                    left_items.push(mvalue);
                }
                Value::Array(left_items)
            } else {
                Value::Array(right_items)
            }
        }
        (Value::Table(mut left_map), Value::Table(right_map)) => {
            if merge_depth > 0 {
                for (rname, rvalue) in right_map {
                    match left_map.remove(&rname) {
                        Some(lvalue) => {
                            let merged_value = merge_toml_values(lvalue, rvalue, merge_depth - 1);
                            left_map.insert(rname, merged_value);
                        }
                        None => {
                            left_map.insert(rname, rvalue);
                        }
                    }
                }
                Value::Table(left_map)
            } else {
                Value::Table(right_map)
            }
        }
        // Catch everything else we didn't handle, and use the right value
        (_, value) => value,
    }
}

#[cfg(test)]
mod test_toml_collation {
    use super::{merge_toml_values, Configuration};

    // NOTE PartialEq for toml::Value is (understandably) order sensitive over array elements, so
    // we convert to `topiary::Configuration` for equality testing. Technically this means our
    // collation tests are not completely general, but that's not really important.

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

        // TODO Update function call for respective collation mode
        let merged: Configuration = merge_toml_values(base, graft, 3).try_into().unwrap();

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

        // TODO Update function call for respective collation mode
        let revised: Configuration = merge_toml_values(base, graft, 3).try_into().unwrap();

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
