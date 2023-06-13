use serde_toml_merge::merge;
use std::env::current_dir;
use topiary::{default_configuration_toml, Configuration};

pub fn parse_configuration() -> Configuration {
    user_lang_toml()
        .expect("TODO: Error")
        .try_into()
        .expect("TODO: Error")
}

/// User configured languages.toml file, merged with the default config.
fn user_lang_toml() -> Result<toml::Value, toml::de::Error> {
    let config = [current_dir().unwrap().join(".topiary")]
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
