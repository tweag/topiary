use crate::{FormatterError, FormatterResult};

/// The languages that we support with query files.
#[derive(Clone, Copy, Debug)]
pub enum Language {
    Json,
    Ocaml,
    Rust,
}

impl Language {
    pub fn new(s: &str) -> FormatterResult<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Language::Json),
            "ocaml" => Ok(Language::Ocaml),
            "rust" => Ok(Language::Rust),
            _ => Err(FormatterError::Query(
                format!("Unsupported language specified: '{s}'"),
                None,
            )),
        }
    }
}
