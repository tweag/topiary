//! This module defines all errors that might be propagated out of the library,
//! including all of the trait implementations one might expect for Errors.

use std::{error::Error, fmt, io, ops::Deref, str, string};

use miette::{Diagnostic, NamedSource, SourceSpan};
use topiary_tree_sitter_facade::{Point, QueryError, Range};

use crate::tree_sitter::NodeSpan;

/// The various errors the formatter may return.
#[derive(Debug)]
pub enum FormatterError {
    /// The input produced output that isn't idempotent, i.e. formatting the
    /// output again made further changes. If this happened using our provided
    /// query files, it is a bug. Please log an issue.
    Idempotence,

    /// The input produced invalid output, i.e. formatting the output again led
    /// to a parsing error. If this happened using our provided query files, it
    /// is a bug. Please log an issue.
    IdempotenceParsing(Box<FormatterError>),

    /// An internal error occurred. This is a bug. Please log an issue.
    Internal(String, Option<Box<dyn Error>>),

    // Tree-sitter could not parse the input without errors.
    Parsing(Box<NodeSpan>),
    /// The query contains a pattern that had no match in the input file.
    PatternDoesNotMatch,

    /// There was an error in the query file. If this happened using our
    /// provided query files, it is a bug. Please log an issue.
    Query(String, Option<QueryError>),

    /// I/O-related errors
    Io(IoError),
}

/// A subtype of `FormatterError::Io`
#[derive(Debug)]
pub enum IoError {
    // NOTE: Filesystem-based IO errors _ought_ to become a thing of the past,
    // once the library and binary code have been completely separated (see
    // Issue #303).
    /// A filesystem based IO error, with an additional owned string to provide
    /// Topiary specific information
    Filesystem(String, io::Error),

    /// Any other Error with an additional owned string to provide Topiary
    /// specific information
    Generic(String, Option<Box<dyn Error>>),
}

impl FormatterError {
    fn get_span(&mut self) -> Option<&mut NodeSpan> {
        match self {
            Self::Parsing(span) => Some(span),
            Self::IdempotenceParsing(err) => err.get_span(),
            _ => None,
        }
    }
    pub fn with_content(mut self, content: String) -> Self {
        if let Some(span) = self.get_span() {
            span.set_content(content);
        }
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        if let Some(span) = self.get_span() {
            span.set_location(location);
        }
        self
    }
}

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let please_log_message = "If this happened with the built-in query files, it is a bug. It would be\nhelpful if you logged this error at\nhttps://github.com/tweag/topiary/issues/new?assignees=&labels=type%3A+bug&template=bug_report.md";
        match self {
            Self::Idempotence => {
                write!(
                    f,
                    "The formatter did not produce the same\nresult when invoked twice (idempotence check).\n\n{please_log_message}"
                )
            }

            Self::IdempotenceParsing(_) => {
                write!(
                    f,
                    "The formatter produced invalid output and\nfailed when trying to format twice (idempotence check).\n\n{please_log_message}\n\nThe following is the error received when running the second time, but note\nthat any line and column numbers refer to the formatted code, not the\noriginal input. Run Topiary with the --skip-idempotence flag to see this\ninvalid formatted code."
                )
            }

            Self::Parsing(span) => {
                let report = miette::Report::new(ErrorSpan::from(span));
                write!(f, "{report:?}")
            }

            Self::PatternDoesNotMatch => {
                write!(
                    f,
                    "The query contains a pattern that does not match the input"
                )
            }

            Self::Internal(message, _)
            | Self::Query(message, _)
            | Self::Io(IoError::Filesystem(message, _) | IoError::Generic(message, _)) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Error for FormatterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Idempotence
            | Self::Parsing(_)
            | Self::PatternDoesNotMatch
            | Self::Io(IoError::Generic(_, None)) => None,
            Self::Internal(_, source) => source.as_ref().map(Deref::deref),
            Self::Query(_, source) => source.as_ref().map(|e| e as &dyn Error),
            Self::Io(IoError::Filesystem(_, source)) => Some(source),
            Self::Io(IoError::Generic(_, Some(source))) => Some(source.as_ref()),
            Self::IdempotenceParsing(source) => Some(source),
        }
    }
}

// pub struct QueryError {
//     pub row: usize,
//     pub column: usize,
//     pub offset: usize,
//     pub message: String,
//     pub kind: QueryErrorKind,
// }
//
//#[derive(Debug)]
// pub struct NodeSpan {
//     pub(crate) range: Range,
//     // source code contents
//     pub content: Option<String>,
//     // source code location
//     pub location: Option<String>,
//     pub language: &'static str,
// }

impl From<&tree_sitter::QueryError> for NodeSpan {
    fn from(e: &tree_sitter::QueryError) -> Self {
        let start_point = Point::new(e.row, e.column);
        let end_point = Point::new(e.row + 1, 1);
        let range = Range::new(e.offset, e.offset + 1, &start_point, &end_point);
        Self {
            range,
            content: None,
            location: None,
            language: "tree_sitter_query",
        }
    }
}

impl From<QueryError> for FormatterError {
    fn from(e: io::Error) -> Self {
        IoError::from(e).into()
    }
}

// NOTE: Filesystem-based IO errors _ought_ to become a thing of the past, once
// the library and binary code have been completely separated (see Issue #303).
impl From<io::Error> for FormatterError {
    fn from(e: io::Error) -> Self {
        IoError::from(e).into()
    }
}

impl From<io::Error> for IoError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => IoError::Filesystem("File not found".into(), e),
            _ => IoError::Filesystem("Could not read or write to file".into(), e),
        }
    }
}
impl From<IoError> for FormatterError {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}

impl From<str::Utf8Error> for FormatterError {
    fn from(e: str::Utf8Error) -> Self {
        Self::Io(IoError::Generic(
            "Input is not valid UTF-8".into(),
            Some(Box::new(e)),
        ))
    }
}

impl From<string::FromUtf8Error> for FormatterError {
    fn from(e: string::FromUtf8Error) -> Self {
        Self::Io(IoError::Generic(
            "Input is not valid UTF-8".into(),
            Some(Box::new(e)),
        ))
    }
}

impl From<fmt::Error> for FormatterError {
    fn from(e: fmt::Error) -> Self {
        Self::Io(IoError::Generic(
            "Failed to format output".into(),
            Some(Box::new(e)),
        ))
    }
}

// We only have to deal with io::BufWriter<Vec<u8>>, but the genericised code is
// clearer
impl<W> From<io::IntoInnerError<W>> for FormatterError
where
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn from(e: io::IntoInnerError<W>) -> Self {
        Self::Io(IoError::Generic(
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

impl From<topiary_tree_sitter_facade::LanguageError> for FormatterError {
    fn from(e: topiary_tree_sitter_facade::LanguageError) -> Self {
        Self::Internal(
            "Error while loading language grammar".into(),
            Some(Box::new(e)),
        )
    }
}

impl From<topiary_tree_sitter_facade::ParserError> for FormatterError {
    fn from(e: topiary_tree_sitter_facade::ParserError) -> Self {
        Self::Internal("Error while parsing".into(), Some(Box::new(e)))
    }
}

impl From<NodeSpan> for FormatterError {
    fn from(span: NodeSpan) -> Self {
        Self::Parsing(Box::new(span))
    }
}

#[derive(Diagnostic, Debug)]
struct ErrorSpan {
    #[source_code]
    src: NamedSource<String>,
    #[label("(ERROR) node")]
    span: SourceSpan,
    range: Range,
}

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.range.start_point();
        let end = self.range.end_point();
        write!(
            f,
            "Parsing error between line {}, column {} and line {}, column {}",
            start.row(),
            start.column(),
            end.row(),
            end.column()
        )
    }
}

impl std::error::Error for ErrorSpan {}

impl From<&Box<NodeSpan>> for ErrorSpan {
    fn from(span: &Box<NodeSpan>) -> Self {
        Self {
            src: NamedSource::new(
                span.location.clone().unwrap_or_default(),
                span.content.clone().unwrap_or_default(),
            )
            .with_language(span.language),
            span: span.source_span(),
            range: span.range,
        }
    }
}
