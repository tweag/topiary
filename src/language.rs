use std::path::{Path, PathBuf};

use crate::{FormatterError, FormatterResult};

/// The languages that we support with query files.
#[derive(Clone, Copy, Debug)]
pub enum Language {
    Json,
    Ocaml,
    Rust,
    Toml,
}

// NOTE This list of extension mappings is influenced by Wilfred Hughes' Difftastic
// https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
const EXTENSIONS: &[(&str, &[&str])] = &[
    (
        "json",
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
    ("ocaml", &["ml"]),
    ("rust", &["rs"]),
    ("toml", &["toml"]),
];

impl Language {
    pub fn new(s: &str) -> FormatterResult<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Language::Json),
            "ocaml" => Ok(Language::Ocaml),
            "rust" => Ok(Language::Rust),
            "toml" => Ok(Language::Toml),
            _ => Err(FormatterError::Query(
                format!("Unsupported language specified: '{s}'"),
                None,
            )),
        }
    }

    pub fn detect(filename: &str) -> FormatterResult<&str> {
        // NOTE This extension search is influenced by Wilfred Hughes' Difftastic
        // https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
        if let Some(extension) = Path::new(filename).extension() {
            let extension = extension.to_str().unwrap();

            for (language, extensions) in EXTENSIONS {
                if extensions.iter().any(|&candidate| candidate == extension) {
                    return Ok(*language);
                }
            }

            return Err(FormatterError::LanguageDetection(
                filename.into(),
                Some(extension.into()),
            ));
        }

        Err(FormatterError::LanguageDetection(filename.into(), None))
    }

    pub fn query_path(language: &str) -> FormatterResult<PathBuf> {
        // Check for support
        Language::new(language)?;

        Ok(
            PathBuf::from(option_env!("TOPIARY_LANGUAGE_DIR").unwrap_or("languages"))
                .join(format!("{language}.scm")),
        )
    }
}
