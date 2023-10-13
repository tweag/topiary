//! Configuration for Topiary can be sourced from either that which is built-in, for from disk.

use std::{env::current_dir, fmt, path::PathBuf};

use directories::ProjectDirs;

use crate::{configuration::serde::Serialisation, error::TopiaryError};

/// Sources of TOML configuration
#[derive(Debug)]
pub enum ConfigSource {
    Builtin,
    File(PathBuf),

    // This is a sentinel element for files that don't exist
    Missing,
}

impl ConfigSource {
    /// Return the valid sources of configuration, in priority order (lowest to highest):
    ///
    /// 1. Built-in configuration (per `Serialisation::default_toml()`)
    /// 2. `~/.config/topiary/languages.toml` (or equivalent)
    /// 3. `.topiary/languages.toml` (or equivalent)
    /// 4. `file`, passed as a CLI argument/environment variable
    pub fn fetch(file: &Option<PathBuf>) -> Vec<Self> {
        [
            Self::Builtin,
            Some(find_os_configuration_dir()).into(),
            find_workspace_configuration_dir().into(),
            file.clone().into(),
        ]
        .into_iter()
        .filter(ConfigSource::is_valid)
        .collect()
    }

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
            ConfigSource::Builtin => Ok(Serialisation::default_toml()),

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
