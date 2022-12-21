use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{FormatterError, FormatterResult};

/// The languages that we support with query files.
#[derive(Clone, Copy, Debug)]
pub enum Language {
    Bash,
    Json,
    Ocaml,
    Rust,
    Toml,
}

// NOTE This list of extension mappings is influenced by Wilfred Hughes' Difftastic
// https://github.com/Wilfred/difftastic/blob/master/src/parse/guess_language.rs
const EXTENSIONS: &[(&str, &[&str])] = &[
    ("bash", &["sh", "bash"]),
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
            "bash" => Ok(Language::Bash),
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

    pub fn query_path(language: &str) -> FormatterResult<PathBuf> {
        // Check for support
        Language::new(language)?;

        Ok(
            PathBuf::from(option_env!("TOPIARY_LANGUAGE_DIR").unwrap_or("languages"))
                .join(format!("{language}.scm")),
        )
    }
}
