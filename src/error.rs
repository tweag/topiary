use std::error::Error;
use std::fmt;
use std::io;
use std::str;
use std::string;

#[derive(Debug)]
pub enum FormatterError {
    Idempotence,
    Internal(String, Option<io::Error>),
    Query(String, Option<tree_sitter::QueryError>),
    Reading(ReadingError),
    Writing(WritingError),
}

#[derive(Debug)]
pub enum ReadingError {
    Io(String, io::Error),
    Utf8(str::Utf8Error),
}

#[derive(Debug)]
pub enum WritingError {
    Fmt(fmt::Error),
    IntoInner(io::IntoInnerError<io::BufWriter<Vec<u8>>>),
    Io(io::Error),
    FromUtf8(string::FromUtf8Error),
}

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Idempotence => {
                write!(
                    f,
                    "The formatter is not idempotent on this input. Please log an error."
                )
            }
            Self::Reading(ReadingError::Io(message, _)) => {
                write!(f, "{message}")
            }
            Self::Reading(ReadingError::Utf8(_)) => {
                write!(f, "Input is not UTF8")
            }
            Self::Writing(_) => {
                write!(f, "Writing error")
            }
            Self::Internal(message, _) | Self::Query(message, _) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Error for FormatterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Idempotence => None,
            Self::Internal(_, source) => source.as_ref().map(|e| e as &dyn Error),
            Self::Query(_, source) => source.as_ref().map(|e| e as &dyn Error),
            Self::Reading(ReadingError::Io(_, source)) => Some(source),
            Self::Reading(ReadingError::Utf8(source)) => Some(source),
            Self::Writing(WritingError::Fmt(source)) => Some(source),
            Self::Writing(WritingError::FromUtf8(source)) => Some(source),
            Self::Writing(WritingError::IntoInner(source)) => Some(source),
            Self::Writing(WritingError::Io(source)) => Some(source),
        }
    }
}
