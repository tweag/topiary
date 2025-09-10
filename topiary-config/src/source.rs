//! Configuration for Topiary can be sourced from either that which is built-in, or from disk.

use std::{env::current_dir, ffi::OsString, fmt, io::Cursor, path::PathBuf};

use crate::error::{TopiaryConfigError, TopiaryConfigResult};

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
    /// Iterate through valid sources of configuration, in priority order (highest to lowest):
    ///
    /// 1. `file`, passed as a CLI argument/environment variable
    /// 2. `.topiary/languages.ncl` (or equivalent)
    /// 3. `~/.config/topiary/languages.ncl`
    /// 4. OS configuration directory (if different from #3)
    /// 5. Built-in configuration: [`Self::builtin_nickel`]
    pub fn config_sources(
        file: &Option<PathBuf>,
    ) -> impl Iterator<Item = (&'static str, Option<Self>)> {
        println!("{:?}", find_os_configuration_dir_config());
        [
            ("CLI", file.clone()),
            ("workspace", find_workspace_configuration_dir_config()),
            // Certain platforms have alternate config directories (macOS)
            // polyfill for linux-like `find_os_configuration_dir_config()`
            #[cfg(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "ios",
                target_arch = "wasm32"
            ))]
            (
                "home",
                std::env::home_dir().map(|home| home.join(".config/topiary/languages.ncl")),
            ),
            (
                "OS",
                Some(find_os_configuration_dir_config().join("languages.ncl")),
            ),
        ]
        .into_iter()
        // If a provided path is a directory, attach `languages.ncl` to the end of the `PathBuf`.
        .map(|(hint, candidate)| {
            let candidate = candidate.map(|mut path| {
                if path.is_dir() {
                    path = path.join("languages.ncl");
                }

                Self::File(path)
            });
            (hint, candidate)
        })
        // add built-in config last
        .chain(std::iter::once(("built-in", Some(Self::Builtin))))
    }

    // return an iterator containing all config sources that have been shown to exist
    fn valid_config_sources(file: &Option<PathBuf>) -> impl Iterator<Item = (&'static str, Self)> {
        Self::config_sources(file).filter_map(|(hint, candidate)| {
            let candidate = candidate?;
            if !candidate.exists() {
                log::debug!("configuration file not found: {}.", candidate);
                return None;
            }

            Some((hint, candidate))
        })
    }
    /// Return all valid configuration sources.
    /// See [`Self::config_sources`].
    pub fn fetch_all(file: &Option<PathBuf>) -> Vec<Self> {
        // We always include the built-in configuration, as a fallback
        log::info!("Adding built-in configuration to merge");
        Self::valid_config_sources(file)
            .inspect(|(hint, candidate)| {
                let Self::File(path) = candidate else { return };
                log::info!(
                    "Adding {hint}-specified configuration to merge: {}",
                    path.to_string_lossy()
                );
            })
            .map(|(_, s)| s)
            .collect()
    }

    fn exists(&self) -> bool {
        match self {
            Source::Builtin => true,
            Source::File(path) => path.exists(),
        }
    }

    /// Return a valid source of configuration with the highest priority.
    /// See [`Self::config_sources`].
    pub fn fetch_one(file: &Option<PathBuf>) -> Self {
        let (hint, source) = Self::valid_config_sources(file)
            .next()
            .expect("built-in should always be present");
        log::info!("Using {hint}-specified configuration: {source}");
        source
    }

    #[allow(clippy::result_large_err)]
    pub fn read(&self) -> TopiaryConfigResult<Vec<u8>> {
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
            Self::Builtin => write!(f, "<built-in>"),

            Self::File(path) => {
                // If the configuration is provided through a file, then we know by this point that
                // it must exist and so the call to `canonicalize` will succeed. However, special
                // cases -- such as process substitution, which creates a temporary FIFO -- may
                // fail if the shell has cleaned things up from under us; in which case, we
                // fallback to the original `path`.
                let config = path.canonicalize().unwrap_or(path.clone());
                write!(f, "{}", config.display())
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
