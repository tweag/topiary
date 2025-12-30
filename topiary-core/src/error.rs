//! This module defines all errors that might be propagated out of the library,
//! including all of the trait implementations one might expect for Errors.

use std::{error::Error, fmt, io, ops::Deref, str, string};

use miette::{Diagnostic, NamedSource, SourceSpan};
use rootcause::{
    Report, ReportConversion,
    markers::{self, Local, SendSync},
    prelude::*,
};
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
    IdempotenceParsing,

    /// An internal error occurred. This is a bug. Please log an issue.
    Internal(String),

    // Tree-sitter could not parse the input without errors.
    Parsing(Box<NodeSpan>),
    /// The query contains a pattern that had no match in the input file.
    PatternDoesNotMatch,

    /// There was an error in the query file. If this happened using our
    /// provided query files, it is a bug. Please log an issue.
    Query(String),

    /// I/O-related errors
    Io(String),
}

// Using context_transform to preserve report structure
// impl<T> ReportConversion<io::Error, markers::Mutable, T> for FormatterError
// where
//     Self: markers::ObjectMarkerFor<T>,
// {
//     fn convert_report(
//         report: Report<io::Error, markers::Mutable, T>,
//     ) -> Report<Self, markers::Mutable, T> {
//         report.context(Self::Io)
//     }
// }

// impl FormatterError {
//     fn get_span(&mut self) -> Option<&mut NodeSpan> {
//         match self {
//             Self::Parsing(span) => Some(span),
//             Self::IdempotenceParsing(err) => err.get_span(),
//             _ => None,
//         }
//     }
//     pub fn with_content(mut self, content: String) -> Self {
//         if let Some(span) = self.get_span() {
//             span.set_content(content);
//         }
//         self
//     }
//
//     pub fn with_location(mut self, location: String) -> Self {
//         if let Some(span) = self.get_span() {
//             span.set_location(location);
//         }
//         self
//     }
// }

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

            Self::IdempotenceParsing => {
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

            Self::Internal(message) | Self::Query(message) | Self::Io(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl Error for FormatterError {}

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

// impl From<&tree_sitter::QueryError> for NodeSpan {
//     fn from(e: &tree_sitter::QueryError) -> Self {
//         let start_point = Point::new(e.row, e.column);
//         let end_point = Point::new(e.row + 1, 1);
//         let range = Range::new(e.offset, e.offset + 1, &start_point, &end_point);
//         Self {
//             range,
//             content: None,
//             location: None,
//             language: "tree_sitter_query",
//         }
//     }
// }

// impl From<io::Error> for IoError {
//     fn from(e: io::Error) -> Self {
//         match e.kind() {
// io::ErrorKind::NotFound => IoError::Filesystem("File not found".into(), e),
// _ => IoError::Filesystem("Could not read or write to file".into(), e),
//         }
//     }
// }

macro_rules! report_conversion {
    ($from:path, $context:expr) => {
        impl<T> ReportConversion<$from, markers::Mutable, T> for FormatterError
        where
            Self: markers::ObjectMarkerFor<T>,
        {
            fn convert_report(
                report: Report<$from, markers::Mutable, T>,
            ) -> Report<Self, markers::Mutable, T> {
                report.context($context)
            }
        }
    };
}

report_conversion!(
    std::str::Utf8Error,
    FormatterError::Io("Input is not valid UTF-8".to_string())
);

report_conversion!(
    std::string::FromUtf8Error,
    FormatterError::Io("Input is not valid UTF-8".to_string())
);

report_conversion!(
    std::fmt::Error,
    FormatterError::Io("Failed to format output".to_string())
);

report_conversion!(
    serde_json::Error,
    FormatterError::Io("Could not serialise JSON output".to_string())
);

report_conversion!(
    topiary_tree_sitter_facade::LanguageError,
    FormatterError::Io("Error while loading language grammar".to_string())
);

report_conversion!(
    topiary_tree_sitter_facade::ParserError,
    FormatterError::Io("Error while parsing".to_string())
);

// We only have to deal with io::BufWriter<Vec<u8>>, but the genericised code is
// clearer
impl<W, T> ReportConversion<io::IntoInnerError<W>, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
    W: io::Write + fmt::Debug + Send + 'static,
{
    fn convert_report(
        report: Report<io::IntoInnerError<W>, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        report.context(Self::Io("Cannot flush internal buffer".to_string()))
    }
}

impl<T> ReportConversion<io::Error, markers::Mutable, T> for FormatterError
where
    Self: markers::ObjectMarkerFor<T>,
{
    fn convert_report(
        report: Report<io::Error, markers::Mutable, T>,
    ) -> Report<Self, markers::Mutable, T> {
        let msg = match report.current_context().kind() {
            io::ErrorKind::NotFound => "File not found",
            _ => "Could not read or write to file",
        };

        report.context(Self::Io(msg.to_string()))
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
