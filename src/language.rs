use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde_derive::{Deserialize, Serialize};

use crate::grammar::{GrammarSource, DYLIB_EXTENSION};
use crate::project_dirs::TOPIARY_DIRS;
use crate::{TopiaryError, TopiaryResult};

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

        if let Some(extension) = extension {
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

    pub fn indent_level(&self) -> isize {
        self.indent_level.unwrap_or(2)
    }

    /// Returns the path to the .o file containing the parser of the language
    pub fn parser_path(&self) -> TopiaryResult<PathBuf> {
        let mut path = TOPIARY_DIRS.cache_dir().to_path_buf();
        path.push("parsers/");
        path.push(format!("{}/parser.{}", self.name, DYLIB_EXTENSION));
        Ok(path)
    }

    pub fn query_path(&self) -> TopiaryResult<PathBuf> {
        Ok(
            PathBuf::from(option_env!("TOPIARY_LANGUAGE_DIR").unwrap_or("languages"))
                .join(format!("{}.scm", self.name)),
        )
    }

    pub fn ensure_available(&self) -> Result<(), TopiaryError> {
        self.grammar.ensure_available(&self.name)
    }

    pub fn get_tree_sitter_language(&self) -> TopiaryResult<tree_sitter::Language> {
        use libloading::{Library, Symbol};
        let library_path = self.parser_path()?;

        let library =
            unsafe { Library::new(&library_path) }.map_err(|e| TopiaryError::ParserLoading(e))?;
        let language_fn_name = format!("tree_sitter_{}", self.name.replace('-', "_"));
        let language = unsafe {
            let language_fn: Symbol<unsafe extern "C" fn() -> tree_sitter::Language> = library
                .get(language_fn_name.as_bytes())
                .map_err(|e| TopiaryError::ParserLoading(e))?;
            language_fn()
        };
        std::mem::forget(library);
        Ok(language)
    }

    #[cfg(test)]
    pub fn dummy_json_lanuage() -> Self {
        Self {
            name: String::from("json"),
            grammar: todo!(),
            extensions: vec![String::from(".json")],
            indent_level: Some(2),
        }
    }
}
