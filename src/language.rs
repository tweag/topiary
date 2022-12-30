use std::ffi::OsString;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};

use crate::grammar::GrammarSource;
use crate::project_dirs::TOPIARY_DIRS;
use crate::{FormatterError, FormatterResult};

/// The languages that we support with query files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub grammar: GrammarSource,
    pub extensions: Vec<String>,
    indent_level: Option<isize>,
}

impl Language {
    pub fn check_extension(&self, filename: &str) -> bool {
        let filename: String = filename.into();
        let extension: Option<OsString> = Path::new(&filename)
            .extension()
            .map(|ext| ext.to_os_string());

        if extension.is_some() {
            let extension = extension.as_deref().unwrap();

            // NOTE This extension search is influenced by Wilfred Hughes' Difftastic
            // https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
            if self
                .extensions
                .iter()
                .any(|candidate| candidate.as_str() == extension)
            {
                return true;
            }
        }

        false
    }

    /// Returns the path to the directory containing the grammar
    pub fn grammar_path(&self) -> FormatterResult<PathBuf> {
        match &self.grammar {
            GrammarSource::Local { path } => Ok(PathBuf::from(path)),
            GrammarSource::Git { remote, revision } => {
                let mut path = TOPIARY_DIRS.cache_dir().to_path_buf();
                path.push("grammars/");
                path.push(format!("{}/", self.name));
                Ok(path)
            }
        }
    }

    pub fn indent_level(&self) -> isize {
        self.indent_level.unwrap_or(2)
    }

    /// Returns the path to the .o file containing the parser of the language
    pub fn parser_path(&self) -> FormatterResult<PathBuf> {
        let mut path = TOPIARY_DIRS.cache_dir().to_path_buf();
        path.push("parsers/");
        path.push(format!("{}.o", self.name));
        Ok(path)
    }

    pub fn query_path(&self) -> FormatterResult<PathBuf> {
        Ok(
            PathBuf::from(option_env!("TOPIARY_LANGUAGE_DIR").unwrap_or("languages"))
                .join(format!("{}.scm", self.name)),
        )
    }

    #[cfg(test)]
    pub fn dummy_json_lanuage() -> Self {
        Self {
            name: "json",
            grammar: todo!(),
            extensions: vec![".json"],
            indent_level: Some(2),
        }
    }
}
