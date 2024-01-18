use std::{error, fmt, io, path, result};

pub type TopiaryConfigResult<T> = result::Result<T, TopiaryConfigError>;

#[derive(Debug)]
pub enum TopiaryConfigError {
    FileNotFound(path::PathBuf),
    ParsingError(toml::de::Error),
    CollationParsingError(String),
    #[cfg(not(wasm))]
    UnknownLanguage(String),
    UnknownExtension(String),
    NoExtension(path::PathBuf),
    #[cfg(not(wasm))]
    QueryFileNotFound(path::PathBuf),
    #[cfg(not(wasm))]
    IoError(io::Error),
    Missing,
    TreeSitterFacade(tree_sitter_facade::LanguageError),
}

impl fmt::Display for TopiaryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopiaryConfigError::FileNotFound(path) => write!(f, "We tried to find your configuration file at {}, but failed to do so. Make sure the file exists.", path.to_string_lossy()),
            TopiaryConfigError::ParsingError(error) => write!(f, "We could not parse the specified configuration file. Our toml parser produced the following error: {error}"),
            TopiaryConfigError::CollationParsingError(arg) => write!(f, "We tried to parse the provided collation mode \"{arg}\", but were unable to do so"),
            TopiaryConfigError::UnknownLanguage(lang) => write!(f, "You were looking for language \"{lang}\", but we do not know that language."),
            TopiaryConfigError::UnknownExtension(ext) => write!(f, "You tried to format a file with extension: \"{ext}\", but we do not know that extension. Make sure the extension is in your configuration file!"),
            TopiaryConfigError::NoExtension(path) => write!(f, "You tried to format {} without specifying a language, but we cannot automatically detect the language because we can't find the filetype extension.", path.to_string_lossy()),
            TopiaryConfigError::QueryFileNotFound(path) => write!(f, "We could not find the query file: \"{}\" anywhere. If you use the TOPIARY_LANGUAGE_DIR environment variable, make sure it set set correctly.", path.to_string_lossy()),
            TopiaryConfigError::IoError(error) => write!(f, "We encountered an io error: {error}"),
            TopiaryConfigError::Missing => write!(f, "A configuration file is missing. If you passed a configuration file, make sure it exists."),
            TopiaryConfigError::TreeSitterFacade(_) => write!(f, "We could not load the grammar for the given language"),
        }
    }
}

impl From<toml::de::Error> for TopiaryConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::ParsingError(e)
    }
}

impl From<io::Error> for TopiaryConfigError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<tree_sitter_facade::LanguageError> for TopiaryConfigError {
    fn from(e: tree_sitter_facade::LanguageError) -> Self {
        Self::TreeSitterFacade(e)
    }
}

impl error::Error for TopiaryConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TopiaryConfigError::ParsingError(e) => e.source(),
            #[cfg(not(wasm))]
            TopiaryConfigError::IoError(e) => e.source(),
            _ => None,
        }
    }
}
