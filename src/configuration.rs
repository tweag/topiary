//! This file deals with reading and parsing the configuration file. It is also
//! responsible for combining the builtin one with the one provided by the user.
use std::path::{Path, PathBuf};

use crate::{language::Language, project_dirs::TOPIARY_DIRS, TopiaryError, TopiaryResult};

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub language: Vec<Language>,
}

impl Configuration {
    pub fn parse() -> TopiaryResult<Self> {
        let config_path: PathBuf = TOPIARY_DIRS.config_dir().join("languages.toml");
        println!("config_path = {:#?}", config_path);
        // TODO: error
        let config_str: String = std::fs::read_to_string(config_path).unwrap();
        let config: Self = toml::from_str(&config_str).unwrap();
        for lang in &config.language {
            println!("LANGUAGE: {}", lang.name);
        }
        Ok(config)
    }

    pub fn find_language_by_extension(&self, file_path: &Path) -> &Language {
        // TODO: Error
        let extension_buf = PathBuf::from(file_path);
        let extension = extension_buf.extension().unwrap();
        self.language
            .iter()
            .find(|&l| l.check_extension(&extension.to_str().unwrap()))
            .unwrap()
    }

    pub fn find_language_by_name(&self, name: &str) -> &Language {
        self.language.iter().find(|&l| l.name == name).unwrap()
    }
}
