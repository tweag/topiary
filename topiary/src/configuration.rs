use std::{collections::HashSet, str::from_utf8};

use crate::{language::Language, FormatterError, FormatterResult};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    pub language: Vec<Language>,
}

impl Configuration {
    // TODO: Should be able to take a filepath
    pub fn parse_default_config() -> Self {
        let default_config = include_bytes!("../../languages.toml");
        let default_config = toml::from_str(from_utf8(default_config).unwrap())
            .expect("Could not parse built-in languages.toml");
        default_config
    }

    pub fn known_extensions<'a>(self: Self) -> HashSet<String> {
        let mut res = HashSet::new();
        for lang in self.language {
            res.extend(lang.extensions);
        }
        res
    }

    pub fn get_language<'a, T: AsRef<str>>(&'a self, name: T) -> &'a Language {
        for lang in &self.language {
            if lang.name == name.as_ref() {
                return &lang;
            }
        }
        todo!("ERIN: Error")
    }
}
