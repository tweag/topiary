use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

trait ErrorString: ToString {}

impl<T: ErrorString> From<T> for Error {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl ErrorString for &str {}
impl ErrorString for String {}
impl ErrorString for mdbook::errors::Error {}
impl ErrorString for pulldown_cmark_to_cmark::Error {}
impl ErrorString for semver::Error {}
impl ErrorString for serde_json::Error {}
impl ErrorString for std::fmt::Error {}
