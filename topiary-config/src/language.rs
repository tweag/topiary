//! This module contains the `Language` struct, which represents a language configuration, and
//! associated methods.

use crate::error::TopiaryConfigError;
use crate::error::TopiaryConfigResult;
use std::collections::HashSet;
use std::path::PathBuf;

/// Language definitions, as far as the CLI and configuration are concerned, contain everything
/// needed to configure formatting for that language.
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in the deserialised
    /// configuration and to convert to the respective Tree-sitter grammar
    pub name: String,

    /// The configuration of the language, includes all properties that Topiary
    /// needs to properly format the language
    pub config: LanguageConfiguration,
}

#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
pub struct LanguageConfiguration {
    /// A set of the filetype extensions associated with this language. This enables Topiary to
    /// switch to the right language based on the input filename.
    pub extensions: HashSet<String>,

    /// The indentation string used for this language; defaults to "  " (i.e., two spaces). Any
    /// string can be provided, but in most instances it will be some whitespace (e.g., "    ",
    /// "\t", etc.)
    pub indent: Option<String>,
}

impl Language {
    pub fn new(name: String, config: LanguageConfiguration) -> Self {
        Self { name, config }
    }

    #[cfg(not(wasm))]
    pub fn find_query_file(&self) -> TopiaryConfigResult<PathBuf> {
        let basename = PathBuf::from(match self.name.as_str() {
            #[cfg(feature = "bash")]
            "bash" => "bash",

            #[cfg(feature = "css")]
            "css" => "css",

            #[cfg(feature = "json")]
            "json" => "json",

            #[cfg(feature = "nickel")]
            "nickel" => "nickel",

            #[cfg(feature = "ocaml")]
            "ocaml" => "ocaml",

            #[cfg(feature = "ocaml_interface")]
            "ocaml_interface" => "ocaml",

            #[cfg(feature = "ocamllex")]
            "ocamllex" => "ocamllex",

            #[cfg(feature = "rust")]
            "rust" => "rust",

            #[cfg(feature = "toml")]
            "toml" => "toml",

            #[cfg(feature = "tree_sitter_query")]
            "tree_sitter_query" => "tree-sitter-query",

            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        });

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 4] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Some(PathBuf::from("./topiary-queries/queries")),
            Some(PathBuf::from("../topiary-queries/queries")),
        ];

        potentials
            .into_iter()
            .flatten()
            .map(|path| path.join(format!("{basename}/formatting.scm")))
            .find(|path| path.exists())
            .ok_or_else(|| TopiaryConfigError::QueryFileNotFound(basename))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
        Ok(match self.name.as_str() {
            #[cfg(feature = "bash")]
            "bash" => tree_sitter_bash::language(),

            #[cfg(feature = "css")]
            "css" => tree_sitter_css::language(),

            #[cfg(feature = "json")]
            "json" => tree_sitter_json::language(),

            #[cfg(feature = "nickel")]
            "nickel" => tree_sitter_nickel::language(),

            #[cfg(feature = "ocaml")]
            "ocaml" => tree_sitter_ocaml::language_ocaml(),

            #[cfg(feature = "ocaml_interface")]
            "ocaml_interface" => tree_sitter_ocaml::language_ocaml_interface(),

            #[cfg(feature = "ocamllex")]
            "ocamllex" => tree_sitter_ocamllex::language(),

            #[cfg(feature = "rust")]
            "rust" => tree_sitter_rust::language(),

            #[cfg(feature = "toml")]
            "toml" => tree_sitter_toml::language(),

            #[cfg(feature = "tree_sitter_query")]
            "tree_sitter_query" => tree_sitter_query::language(),

            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        }
        .into())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn grammar(&self) -> TopiaryConfigResult<topiary_tree_sitter_facade::Language> {
        let language_name = match self.name.as_str() {
            #[cfg(feature = "bash")]
            "bash" => "bash",

            #[cfg(feature = "css")]
            "css" => "css",

            #[cfg(feature = "json")]
            "json" => "json",

            #[cfg(feature = "nickel")]
            "nickel" => "nickel",

            #[cfg(feature = "ocaml")]
            "ocaml" => "ocaml",

            #[cfg(feature = "ocaml_interface")]
            "ocaml_interface" => "ocaml_interface",

            #[cfg(feature = "ocamllex")]
            "ocamllex" => "ocamllex",

            #[cfg(feature = "rust")]
            "rust" => "rust",

            #[cfg(feature = "toml")]
            "toml" => "toml",

            #[cfg(feature = "tree_sitter_query")]
            "tree_sitter_query" => "query",

            name => return Err(TopiaryConfigError::UnknownLanguage(name.to_string())),
        };

        Ok(topiary_web_tree_sitter_sys::Language::load_path(&format!(
            "/playground/scripts/tree-sitter-{language_name}.wasm"
        ))
        .await
        .map_err(|e| {
            let error: topiary_tree_sitter_facade::LanguageError = e.into();
            error
        })?
        .into())
    }
}
