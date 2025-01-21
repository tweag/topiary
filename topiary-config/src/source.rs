//! Configuration for Topiary can be sourced from either that which is built-in, or from disk.

use std::{env::current_dir, ffi::OsString, fmt, io::Cursor, path::PathBuf};

use crate::error::TopiaryConfigError;

/// Sources of Nickel configuration
#[derive(Debug, Clone)]
pub enum Source {
    Builtin,
    File(PathBuf),
}

impl From<Source> for nickel_lang_core::program::Input<Cursor<String>, OsString> {
    fn from(source: Source) -> Self {
        match source {
            Source::Builtin => Self::Source(Cursor::new(source.builtin_nickel()), "builtin".into()),
            Source::File(path) => Self::Path(path.into()),
        }
    }
}

impl Source {
    /// Return the valid sources of configuration, in priority order (highest to lowest):
    ///
    /// 1. `file`, passed as a CLI argument/environment variable
    /// 2. `.topiary/languages.ncl` (or equivalent)
    /// 3. `~/.config/topiary/languages.ncl` (or equivalent)
    /// 4. Built-in configuration (per `Self::builtin_nickel()`)
    pub fn fetch_all(file: &Option<PathBuf>) -> Vec<Self> {
        let candidates = [
            Some(find_os_configuration_dir_config()),
            find_workspace_configuration_dir_config(),
            file.clone(),
        ];

        // We always include the built-in configuration, as a fallback
        let mut res: Vec<Self> = vec![Self::Builtin];

        for candidate in candidates {
            if let Some(path) = Self::find(&candidate) {
                res.push(Self::File(path));
            }
        }

        res
    }

    /// Return the source of configuration that has top priority among available ones.
    /// The priority order is, from highest to lowest:
    ///
    /// 1. `file`, passed as a CLI argument/environment variable
    /// 2. `.topiary/languages.ncl` (or equivalent)
    /// 3. `~/.config/topiary/languages.ncl` (or equivalent)
    /// 4. Built-in configuration (per `Self::builtin_nickel()`)
    pub fn fetch_one(file: &Option<PathBuf>) -> Self {
        let cli_specified = Self::find(&file.clone()).map(Self::File);
        let workspace_specified =
            Self::find(&find_workspace_configuration_dir_config()).map(Self::File);
        let os_config_specified =
            Self::find(&Some(find_os_configuration_dir_config())).map(Self::File);

        if let Some(res) = cli_specified {
            log::info!("Using CLI-specified configuration: {res}");
            res
        } else if let Some(res) = workspace_specified {
            log::info!("Using workspace-specified configuration: {res}");
            res
        } else if let Some(res) = os_config_specified {
            log::info!("Using global os-specified configuration: {res}");
            res
        } else {
            log::info!("Using built-in configuration");
            Self::Builtin
        }
    }

    /// Attempts to find a configuration file, given a `path` parameter. If `path` is `None`, then
    /// the function returns `None`.
    /// Otherwise, if the path is a directory, then it attempts to find a `languages.ncl` file
    /// within that directory. If the file exists, then it returns `Some(path.join("languages.ncl"))`.
    /// If the file does not exist, then it logs a message and returns `None`. If the path is a file,
    /// then it returns `Some(path)`.
    fn find(path: &Option<PathBuf>) -> Option<PathBuf> {
        match path {
            None => None,
            Some(path) => {
                let candidate = if path.is_dir() {
                    path.join("languages.ncl")
                } else {
                    path.clone()
                };

                if candidate.exists() {
                    Some(candidate)
                } else {
                    log::info!(
                        "Could not find configuration file: {}. Defaulting to built-in configuration.",
                        candidate.to_string_lossy()
                    );
                    None
                }
            }
        }
    }

    pub fn read(&self) -> Result<Vec<u8>, TopiaryConfigError> {
        match self {
            Self::Builtin => Ok(self.builtin_nickel().into_bytes()),
            Self::File(path) => std::fs::read_to_string(path)
                .map_err(TopiaryConfigError::Io)
                .map(|s| s.into_bytes()),
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
    crate::project_dirs().config_dir().to_path_buf()
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
