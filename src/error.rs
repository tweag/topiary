use std::error::Error;
use std::fmt;
use std::io;
use std::io::BufWriter;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum FormatterError {
    Idempotence,
    Internal(String, Option<io::Error>),
    Query(String, Option<tree_sitter::QueryError>),
    Reading(String, io::Error),
    Writing(WritingError),
}

#[derive(Debug)]
pub enum WritingError {
    Fmt(fmt::Error),
    IntoInner(io::IntoInnerError<BufWriter<Vec<u8>>>),
    Io(io::Error),
    FromUtf8(FromUtf8Error),
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
            Self::Writing(_) => {
                write!(f, "Writing error")
            }
            Self::Internal(message, _) | Self::Query(message, _) | Self::Reading(message, _) => {
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
            Self::Reading(_, source) => Some(source),
            Self::Writing(WritingError::Fmt(source)) => Some(source),
            Self::Writing(WritingError::FromUtf8(source)) => Some(source),
            Self::Writing(WritingError::IntoInner(source)) => Some(source),
            Self::Writing(WritingError::Io(source)) => Some(source),
        }
    }
}
