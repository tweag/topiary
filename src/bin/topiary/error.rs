use std::{error, fmt, io, process::ExitCode, result};
use topiary::FormatterError;

/// A convenience wrapper around `std::result::Result<T, TopiaryError>`.
pub type CLIResult<T> = result::Result<T, TopiaryError>;

/// The errors that can be raised by either the Topiary CLI, or passed through by the formatter
/// library code. This acts as a supertype of `FormatterError`, with additional members to denote
/// CLI-specific failures.
#[derive(Debug)]
pub enum TopiaryError {
    Lib(FormatterError),
    Bin(String, Option<CLIError>),
}

/// A subtype of `TopiaryError::Bin`
#[derive(Debug)]
pub enum CLIError {
    IOError(io::Error),
    Generic(Box<dyn error::Error>),
}

impl fmt::Display for TopiaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lib(error) => write!(f, "{error}"),
            Self::Bin(message, _) => write!(f, "{message}"),
        }
    }
}

impl error::Error for TopiaryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Lib(error) => error.source(),
            Self::Bin(_, Some(CLIError::IOError(error))) => Some(error),
            Self::Bin(_, Some(CLIError::Generic(error))) => error.source(),
            Self::Bin(_, None) => None,
        }
    }
}

impl From<TopiaryError> for ExitCode {
    fn from(e: TopiaryError) -> Self {
        let exit_code = match e {
            // Formatting errors: Exit 8
            TopiaryError::Lib(FormatterError::Formatting(_)) => 8,

            // Idempotency errors: Exit 7
            TopiaryError::Lib(FormatterError::Idempotence) => 7,

            // Language detection errors: Exit 6
            TopiaryError::Lib(FormatterError::LanguageDetection(_, _)) => 6,

            // Parsing errors: Exit 5
            TopiaryError::Lib(FormatterError::Parsing { .. }) => 5,

            // Query errors: Exit 4
            TopiaryError::Lib(FormatterError::Query(_, _)) => 4,

            // I/O errors: Exit 3
            TopiaryError::Lib(FormatterError::Io(_)) => 3,
            TopiaryError::Bin(_, Some(CLIError::IOError(_))) => 3,

            // Bad arguments: Exit 2
            // (Handled by clap: https://github.com/clap-rs/clap/issues/3426)

            // Anything else: Exit 1
            _ => 1,
        };

        ExitCode::from(exit_code)
    }
}

impl From<FormatterError> for TopiaryError {
    fn from(e: FormatterError) -> Self {
        Self::Lib(e)
    }
}

impl From<io::Error> for TopiaryError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => {
                Self::Bin("File not found".into(), Some(CLIError::IOError(e)))
            }

            _ => Self::Bin(
                "Could not read or write to file".into(),
                Some(CLIError::IOError(e)),
            ),
        }
    }
}

impl From<tempfile::PersistError> for TopiaryError {
    fn from(e: tempfile::PersistError) -> Self {
        Self::Bin(
            "Could not persist output to disk".into(),
            Some(CLIError::IOError(e.error)),
        )
    }
}

// We only have to deal with io::BufWriter<crate::output::OutputFile>,
// but the genericised code is clearer
impl<W> From<io::IntoInnerError<W>> for TopiaryError
where
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn from(e: io::IntoInnerError<W>) -> Self {
        Self::Bin(
            "Could not flush internal buffer".into(),
            Some(CLIError::Generic(Box::new(e))),
        )
    }
}
