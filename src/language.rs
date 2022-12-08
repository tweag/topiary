// Portions of this code, highlighted below, are influenced by Difftastic (MIT licensed):
//
// Copyright (c) 2021-2022 Wilfred Hughes
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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

// NOTE Influenced by Difftastic; see above
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
        // NOTE Influenced by Difftastic; see above
        if let Some(extension) = Path::new(filename).extension() {
            let extension = extension.to_str().unwrap();

            for (language, extensions) in EXTENSIONS {
                for candidate in extensions.iter() {
                    if *candidate == extension {
                        return Ok(*language);
                    }
                }
            }

            return Err(FormatterError::Query(
                format!("Cannot detect language from unknown extension: '{filename}'"),
                None,
            ));
        }

        Err(FormatterError::Query(
            format!("Cannot detect language without extension: '{filename}'"),
            None,
        ))
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
