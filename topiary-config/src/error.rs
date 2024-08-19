use std::{error, fmt, io, path, result};

pub type TopiaryConfigResult<T> = result::Result<T, TopiaryConfigError>;

#[derive(Debug)]
pub enum TopiaryConfigError {
    FileNotFound(path::PathBuf),
    UnknownLanguage(String),
    UnknownExtension(String),
    NoExtension(path::PathBuf),
    #[cfg(not(target_arch = "wasm32"))]
    QueryFileNotFound(path::PathBuf),
    IoError(io::Error),
    Missing,
    TreeSitterFacade(topiary_tree_sitter_facade::LanguageError),
    Nickel(nickel_lang_core::error::Error),
    NickelDeserialization(nickel_lang_core::deserialize::RustDeserializationError),
    #[cfg(not(target_arch = "wasm32"))]
    LibLoading(libloading::Error),
    #[cfg(not(target_arch = "wasm32"))]
    Git(git2::Error),
}

impl fmt::Display for TopiaryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopiaryConfigError::FileNotFound(path) => write!(f, "We tried to find your configuration file at {}, but failed to do so. Make sure the file exists.", path.to_string_lossy()),
            TopiaryConfigError::UnknownLanguage(lang) => write!(f, "You were looking for language \"{lang}\", but we do not know that language."),
            TopiaryConfigError::UnknownExtension(ext) => write!(f, "You tried to format a file with extension: \"{ext}\", but we do not know that extension. Make sure the extension is in your configuration file!"),
            TopiaryConfigError::NoExtension(path) => write!(f, "You tried to format {} without specifying a language, but we cannot automatically detect the language because we can't find the filetype extension.", path.to_string_lossy()),
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::QueryFileNotFound(path) => write!(f, "We could not find the query file: \"{}\" anywhere. If you use the TOPIARY_LANGUAGE_DIR environment variable, make sure it set set correctly.", path.to_string_lossy()),
            TopiaryConfigError::IoError(error) => write!(f, "We encountered an io error: {error}"),
            TopiaryConfigError::Missing => write!(f, "A configuration file is missing. If you passed a configuration file, make sure it exists."),
            TopiaryConfigError::TreeSitterFacade(_) => write!(f, "We could not load the grammar for the given language"),
            TopiaryConfigError::Nickel(e) => write!(f, "Nickel error: {:?}", e),
            TopiaryConfigError::NickelDeserialization(e) => write!(f, "Nickel error: {:?}", e),
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::LibLoading(e) => write!(f, "Libloading error: {:?}", e),
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::Git(e) => write!(f, "Git error: {:?}", e),
        }
    }
}

impl From<nickel_lang_core::deserialize::RustDeserializationError> for TopiaryConfigError {
    fn from(e: nickel_lang_core::deserialize::RustDeserializationError) -> Self {
        Self::NickelDeserialization(e)
    }
}

impl From<nickel_lang_core::error::Error> for TopiaryConfigError {
    fn from(e: nickel_lang_core::error::Error) -> Self {
        Self::Nickel(e)
    }
}

impl From<io::Error> for TopiaryConfigError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<topiary_tree_sitter_facade::LanguageError> for TopiaryConfigError {
    fn from(e: topiary_tree_sitter_facade::LanguageError) -> Self {
        Self::TreeSitterFacade(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<libloading::Error> for TopiaryConfigError {
    fn from(e: libloading::Error) -> Self {
        Self::LibLoading(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<git2::Error> for TopiaryConfigError {
    fn from(e: git2::Error) -> Self {
        Self::Git(e)
    }
}

impl error::Error for TopiaryConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            TopiaryConfigError::IoError(e) => e.source(),
            _ => None,
        }
    }
}
