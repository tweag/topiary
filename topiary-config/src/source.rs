//! Configuration for Topiary can be sourced from either that which is built-in, or from disk.

use std::{env::current_dir, fmt, path::PathBuf};

use directories::ProjectDirs;

use crate::error::TopiaryConfigError;

/// Sources of TOML configuration
#[derive(Debug)]
pub enum Source {
    Builtin,
    File(PathBuf),
}

impl Source {
    /// Return the valid sources of configuration, in priority order (lowest to highest):
    ///
    /// 1. Built-in configuration (per `Self::builtin_nickel()`)
    /// 2. `~/.config/topiary/languages.toml` (or equivalent)
    /// 3. `.topiary/languages.toml` (or equivalent)
    /// 4. `file`, passed as a CLI argument/environment variable
    pub fn fetch(file: &Option<PathBuf>) -> Vec<Self> {
        let candidates = [
            Some(find_os_configuration_dir_config()),
            find_workspace_configuration_dir_config(),
            file.clone(),
        ];

        // We always include the built-in configuration, as a fallback
        let mut res: Vec<Self> = vec![Self::Builtin];

        for candiate in candidates {
            if let Some(path) = Self::find(&candiate) {
                res.push(Self::File(path));
            }
        }

        res
    }

    /// Attempts to find a configuration file, given a `path` parameter. If `path` is `None`, then
    /// the function returns `None`.
    /// Otherwise, if the path is a rectory, then it attempts to find a `languages.toml` file
    /// within that directory. If the file exists, then it returns `Some(path.join("languages.toml"))`.
    /// If the file does not exist, then it logs a warning and returns `None`. If the path is a file,
    /// then it returns `Some(path)`.
    fn find(path: &Option<PathBuf>) -> Option<PathBuf> {
        match path {
            None => None,
            Some(path) => {
                let candidate = if path.is_dir() {
                    path.join("languages.toml")
                } else {
                    path.clone()
                };

                if candidate.exists() {
                    Some(candidate)
                } else {
                    log::warn!(
                        "Could not find configuration file: {}. Are you sure it exists?",
                        candidate.to_string_lossy()
                    );
                    None
                }
            }
        }
    }

    pub fn read(&self) -> Result<String, TopiaryConfigError> {
        match self {
            Self::Builtin => Ok(self.builtin_nickel()),
            Self::File(path) => std::fs::read_to_string(path).map_err(TopiaryConfigError::IoError),
        }
    }

    fn builtin_nickel(&self) -> String {
        include_str!("../languages.ncl").to_string()
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin => write!(f, "Built-in configuration"),

            Self::File(path) => {
                // We only stringify the path when we know it exists, so the call to `canonicalize`
                // is safe to unwrap. (All bets are off, if called from elsewhere.)
                write!(f, "{}", path.canonicalize().unwrap().to_string_lossy())
            }
        }
    }
}

/// Find the OS-specific configuration directory
fn find_os_configuration_dir_config() -> PathBuf {
    ProjectDirs::from("", "", "topiary")
        .expect("Could not access the OS's Home directory")
        .config_dir()
        .to_path_buf()
}

/// Ascend the directory hierarchy, starting from the current working directory, in search of the
/// nearest `.topiary` configuration directory
fn find_workspace_configuration_dir_config() -> Option<PathBuf> {
    current_dir()
        .expect("Could not get current working directory")
        .ancestors()
        .map(|path| path.join(".topiary"))
        .find(|path| path.exists())
}
