// use std::env::var;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use crate::{FormatterError, FormatterResult, IoError};

/// The languages that we support with query files.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Language {
    Bash,
    Json,
    Nickel,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    Rust,
    Toml,
}

// NOTE This list of extension mappings is influenced by Wilfred Hughes' Difftastic
// https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
const EXTENSIONS: &[(Language, &[&str])] = &[
    (Language::Bash, &["sh", "bash"]),
    (
        Language::Json,
        &[
            "json",
            "avsc",
            "geojson",
            "gltf",
            "har",
            "ice",
            "JSON-tmLanguage",
            "jsonl",
            "mcmeta",
            "tfstate",
            "tfstate.backup",
            "topojson",
            "webapp",
            "webmanifest",
        ],
    ),
    (Language::Nickel, &["ncl"]),
    (Language::OcamlImplementation, &["ml"]),
    (Language::OcamlInterface, &["mli"]),
    (Language::Rust, &["rs"]),
    (Language::Toml, &["toml"]),
];

impl Language {
    pub fn new(s: &str) -> FormatterResult<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Language::Bash),
            "json" => Ok(Language::Json),
            "nickel" => Ok(Language::Nickel),
            "ocaml" => Ok(Language::Ocaml),
            "ocaml-implementation" => Ok(Language::OcamlImplementation),
            "ocaml-interface" => Ok(Language::OcamlInterface),
            "rust" => Ok(Language::Rust),
            "toml" => Ok(Language::Toml),

            _ => Err(FormatterError::Query(
                format!("Unsupported language specified: '{s}'"),
                None,
            )),
        }
    }

    pub fn detect(filename: &str) -> FormatterResult<Language> {
        let filename: String = filename.into();
        let extension: Option<OsString> = Path::new(&filename)
            .extension()
            .map(|ext| ext.to_os_string());

        if extension.is_some() {
            let extension = extension.as_deref().unwrap();

            // NOTE This extension search is influenced by Wilfred Hughes' Difftastic
            // https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
            for (language, extensions) in EXTENSIONS {
                if extensions.iter().any(|&candidate| candidate == extension) {
                    return Ok(*language);
                }
            }
        }

        Err(FormatterError::LanguageDetection(filename, extension))
    }

    // Different languages may map to the same query file, because their grammars
    // produce similar trees, which can be formatted with the same queries.
    pub fn query_file_base_name(language: Language) -> &'static str {
        match language {
            Language::Bash => "bash",
            Language::Json => "json",
            Language::Nickel => "nickel",
            Language::Ocaml => "ocaml",
            Language::OcamlImplementation => "ocaml",
            Language::OcamlInterface => "ocaml",
            Language::Rust => "rust",
            Language::Toml => "toml",
        }
    }

    pub fn query_path(language: Language) -> FormatterResult<PathBuf> {
        // We test 3 different locations for query files, in the following priority order,
        // returning the first that exists:
        //
        // 1. Under the TOPIARY_LANGUAGE_DIR env variable at runtime;
        // 2. Under the TOPIARY_LANGUAGE_DIR env variable at build time;
        // 3. Under the "./languages" subdirectory.
        //
        // If all of these fail, we return an I/O error.

        let query_file = Self::query_file_base_name(language);

        #[rustfmt::skip]
        let potentials: [Option<PathBuf>; 3] = [
            std::env::var("TOPIARY_LANGUAGE_DIR").map(PathBuf::from).ok(),
            option_env!("TOPIARY_LANGUAGE_DIR").map(PathBuf::from),
            Some(PathBuf::from("./languages")),
        ];

        potentials
            .into_iter()
            .flatten()
            .map(|path| path.join(format!("{query_file}.scm")))
            .find(|path| path.exists())
            .ok_or_else(|| {
                FormatterError::Io(IoError::Filesystem(
                    "Language query file could not be found".into(),
                    io::Error::from(io::ErrorKind::NotFound),
                ))
            })
    }

    pub fn grammars(language: Language) -> Vec<tree_sitter::Language> {
        match language {
            Language::Bash => vec![tree_sitter_bash::language()],
            Language::Json => vec![tree_sitter_json::language()],
            Language::Nickel => vec![tree_sitter_nickel::language()],
            Language::Ocaml => vec![
                tree_sitter_ocaml::language_ocaml(),
                tree_sitter_ocaml::language_ocaml_interface(),
            ],
            Language::OcamlImplementation => vec![tree_sitter_ocaml::language_ocaml()],
            Language::OcamlInterface => vec![tree_sitter_ocaml::language_ocaml_interface()],
            Language::Rust => vec![tree_sitter_rust::language()],
            Language::Toml => vec![tree_sitter_toml::language()],
        }
    }
}
