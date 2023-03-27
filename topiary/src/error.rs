use std::{error::Error, fmt, io, path::PathBuf, str, string};

/// The various errors the formatter may return.
#[derive(Debug)]
pub enum FormatterError {
    /// The input produced output that cannot be formatted, i.e. trying to format the
    /// output again produced an error. If this happened using our provided
    /// query files, it is a bug. Please log an issue.
    Formatting(Box<FormatterError>),

    /// The input produced output that isn't idempotent, i.e. formatting the
    /// output again made further changes. If this happened using our provided
    /// query files, it is a bug. Please log an issue.
    Idempotence,

    /// An internal error occurred. This is a bug. Please log an issue.
    Internal(String, Option<Box<dyn Error>>),

    /// Tree-sitter could not parse the input without errors.
    Parsing {
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    },

    /// The query contains a pattern that had no match in the input file.
    /// Should only be raised in the test suite.
    PatternDoesNotMatch(String),

    /// There was an error in the query file. If this happened using our
    /// provided query files, it is a bug. Please log an issue.
    Query(String, Option<tree_sitter_facade::QueryError>),

    /// Could not detect the input language from the (filename, Option<extension>)
    LanguageDetection(PathBuf, Option<String>),

    /// I/O-related errors
    Io(IoError),
}

/// A subtype of `FormatterError::Io`
#[derive(Debug)]
pub enum IoError {
    // NOTE Filesystem-based IO errors _ought_ to become a thing of the past, once the library and
    // binary code have been completely separated (see Issue #303).
    Filesystem(String, io::Error),
    Generic(String, Option<Box<dyn Error>>),
}

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let please_log_message = "It would be helpful if you logged this error at https://github.com/tweag/topiary/issues/new?assignees=&labels=type%3A+bug&template=bug_report.md";
        match self {
            Self::Idempotence => {
                write!(
                    f,
                    "The formatter did not produce the same result when invoked twice (idempotence check).\n{please_log_message}"
                )
            }

            Self::Parsing {
                start_line,
                start_column,
                end_line,
                end_column,
            } => {
                write!(f, "Parsing error between line {start_line}, column {start_column} and line {end_line}, column {end_column}")
            }

            Self::LanguageDetection(filename, extension) => {
                let file: String = match filename.to_str().unwrap() {
                    "-" => "from standard input".into(),
                    _ => format!("of file '{}'", filename.to_string_lossy()),
                };

                match extension {
                    Some(extension) => write!(f,
                        "Cannot detect language {file} due to unknown extension '.{extension}'. Try specifying language explicitly.",
                    ),
                    None => write!(f,
                        "Cannot detect language {file}. Try specifying language explicitly."
                    ),
                }
            }

            Self::Formatting(_err) => {
                write!(
                    f,
                    "The formatter failed when trying to format the code twice (idempotence check).\nThis probably means that the formatter produced invalid code.\n{please_log_message}"
                )
            }

            Self::PatternDoesNotMatch(pattern_content) => {
                write!(
                    f,
                    "The following pattern matches nothing in the input:\n{pattern_content}"
                )
            }

            Self::Internal(message, _)
            | Self::Query(message, _)
            | Self::Io(IoError::Filesystem(message, _))
            | Self::Io(IoError::Generic(message, _)) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Error for FormatterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Idempotence => None,
            Self::Internal(_, source) => source.as_ref().map(|e| &**e),
            Self::Parsing { .. } => None,
            Self::PatternDoesNotMatch(_) => None,
            Self::Query(_, source) => source.as_ref().map(|e| e as &dyn Error),
            Self::LanguageDetection(_, _) => None,
            Self::Io(IoError::Filesystem(_, source)) => Some(source),
            Self::Io(IoError::Generic(_, Some(source))) => Some(source.as_ref()),
            Self::Io(IoError::Generic(_, None)) => None,
            Self::Formatting(err) => Some(err),
        }
    }
}

// NOTE Filesystem-based IO errors _ought_ to become a thing of the past, once the library and
// binary code have been completely separated (see Issue #303).
impl From<io::Error> for FormatterError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => {
                FormatterError::Io(IoError::Filesystem("File not found".into(), e))
            }

            _ => FormatterError::Io(IoError::Filesystem(
                "Could not read or write to file".into(),
                e,
            )),
        }
    }
}

impl From<str::Utf8Error> for FormatterError {
    fn from(e: str::Utf8Error) -> Self {
        FormatterError::Io(IoError::Generic(
            "Input is not valid UTF-8".into(),
            Some(Box::new(e)),
        ))
    }
}

impl From<string::FromUtf8Error> for FormatterError {
    fn from(e: string::FromUtf8Error) -> Self {
        FormatterError::Io(IoError::Generic(
            "Input is not valid UTF-8".into(),
            Some(Box::new(e)),
        ))
    }
}

impl From<fmt::Error> for FormatterError {
    fn from(e: fmt::Error) -> Self {
        FormatterError::Io(IoError::Generic(
            "Failed to format output".into(),
            Some(Box::new(e)),
        ))
    }
}

// We only have to deal with io::BufWriter<Vec<u8>>, but the genericised code is clearer
impl<W> From<io::IntoInnerError<W>> for FormatterError
where
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn from(e: io::IntoInnerError<W>) -> Self {
        FormatterError::Io(IoError::Generic(
            "Cannot flush internal buffer".into(),
            Some(Box::new(e)),
        ))
    }
}

impl From<serde_json::Error> for FormatterError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal("Could not serialise JSON output".into(), Some(Box::new(e)))
    }
}

impl From<tree_sitter_facade::LanguageError> for FormatterError {
    fn from(e: tree_sitter_facade::LanguageError) -> Self {
        Self::Internal(
            "Error while loading language grammar".into(),
            Some(Box::new(e)),
        )
    }
}

impl From<tree_sitter_facade::ParserError> for FormatterError {
    fn from(e: tree_sitter_facade::ParserError) -> Self {
        Self::Internal("Error while parsing".into(), Some(Box::new(e)))
    }
}
