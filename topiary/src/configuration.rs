use std::{collections::HashSet, str::from_utf8};

use crate::{language::Language, FormatterError, FormatterResult};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
    pub language: Vec<Language>,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration { language: vec![] }
    }

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

    pub fn parse_default_configuration() -> Self {
        default_configuration_toml()
            .try_into()
            .expect("TODO: Error")
    }
}

/// Default built-in languages.toml.
pub fn default_configuration_toml() -> toml::Value {
    let default_config = include_bytes!("../../languages.toml");
    toml::from_str(from_utf8(default_config).unwrap())
        .expect("Could not parse built-in languages.toml to valid toml")
}
