use serde_toml_merge::merge;
use std::{env::current_dir, path::PathBuf};
use topiary::{default_configuration_toml, Configuration};

use crate::error::{CLIResult, TopiaryError};

pub fn parse_configuration() -> CLIResult<Configuration> {
    user_configuration_toml()?
        .try_into()
        .map_err(TopiaryError::from)
}

/// User configured languages.toml file, merged with the default config.
fn user_configuration_toml() -> CLIResult<toml::Value> {
    let config = [find_workspace().join(".topiary")]
        .into_iter()
        .map(|path| path.join("languages.toml"))
        .filter_map(|file| {
            std::fs::read_to_string(file)
                .map(|config| toml::from_str(&config))
                .ok()
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(default_configuration_toml(), |a, b| {
            merge(a, b).expect("TODO: Gracefull fail")
        });

    Ok(config)
}

pub fn find_workspace() -> PathBuf {
    let current_dir = current_dir().expect("Could not get current working directory");
    for ancestor in current_dir.ancestors() {
        if ancestor.join(".topiary").exists() {
            return ancestor.to_owned();
        }
    }

    // Default to the current dir if we could not find an ancestor with the .topiary directory
    // If current_dir does not contain a .topiary, it will be filtered our in the `user_lang_toml` function.
    current_dir
}
