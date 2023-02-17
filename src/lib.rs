//! A general code formatter that relies on
//! [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) for language
//! parsing.
//!
//! In order for a language to be supported, there must be a [Tree-sitter
//! grammar](https://tree-sitter.github.io/tree-sitter/#available-parsers)
//! available, and there must be a query file that dictates how that language is
//! to be formatted. We include query files for some languages.
//!
//! More details can be found on
//! [GitHub](https://github.com/tweag/topiary).

use configuration::Configuration;
pub use error::{FormatterError, IoError};
use itertools::Itertools;
pub use language::Language;
use pretty_assertions::StrComparison;
use std::io;

mod atom_collection;
mod configuration;
mod error;
mod language;
mod pretty;
mod tree_sitter;

/// An atom represents a small piece of the output. We turn Tree-sitter nodes
/// into atoms, and we add white-space atoms where appropriate. The final list
/// of atoms is rendered to the output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Atom {
    /// We don't allow consecutive `Hardline`, but a `Blankline` will render two
    /// newlines to produce a blank line.
    Blankline,
    /// A "no-op" atom that will not produce any output.
    Empty,
    /// Represents a newline.
    Hardline,
    /// Signals the end of an indentation block.
    IndentEnd,
    /// Signals the start of an indentation block. Any lines between the
    /// beginning and the end will be indented. In single-line constructs where
    /// the beginning and the end occurs on the same line, there will be no
    /// indentation.
    IndentStart,
    /// Represents the contents of a named Tree-sitter node. We track the node id here
    /// as well.
    Leaf {
        content: String,
        id: usize,
        // marks the leaf to be printed on a single line, with no indentation
        single_line_no_indent: bool,
    },
    /// Represents a literal string, such as a semicolon.
    Literal(String),
    /// Represents a literal string, such as a semicolon. It will be printed only
    // in multi-line constructs
    MultilineOnlyLiteral(String),
    /// Represents a softline. It will be turned into a hardline for multi-line
    /// constructs, and either a space or nothing for single-line constructs.
    Softline {
        spaced: bool,
    },
    /// Represents a space. Consecutive spaces are reduced to one before rendering.
    Space,
    /// Represents a segment to be deleted.
    // It is a segment, because if one wants to delete a node,
    // it might happen that it contains several leaves.
    DeleteBegin,
    DeleteEnd,
    /// Scoped commands
    // ScopedSoftline works together with the @open_scope and @end_scope query tags.
    // To decide if a scoped softline must be expanded into a hardline, we look at
    // the innermost scope having the corresponding `scope_id`, that encompasses it.
    // We expand the softline if that scope is multi-line.
    // The `id` value is here for technical reasons, it allows tracking of the atom
    // during post-processing.
    ScopedSoftline {
        id: usize,
        scope_id: String,
        spaced: bool,
    },
}

/// A convenience wrapper around `std::result::Result<T, FormatterError>`.
pub type FormatterResult<T> = std::result::Result<T, FormatterError>;

/// The function that takes an input and formats an output.
///
/// # Examples
///
/// ```
/// use std::fs::File;
/// use std::io::BufReader;
/// use topiary::{formatter, FormatterError};
///
/// let input = "[1,2]".to_string();
/// let mut input = input.as_bytes();
/// let mut output = Vec::new();
/// let mut query = BufReader::new(File::open("languages/json.scm").expect("query file"));
///
/// match formatter(&mut input, &mut output, &mut query, None, false) {
///   Ok(()) => {
///     let formatted = String::from_utf8(output).expect("valid utf-8");
///   }
///   Err(FormatterError::Query(message, _)) => {
///     panic!("Error in query file: {message}");
///   }
///   Err(_) => {
///     panic!("An error occurred");
///   }
/// }
/// ```
pub fn formatter(
    input: &mut dyn io::Read,
    output: &mut dyn io::Write,
    query: &mut dyn io::Read,
    language: Option<Language>,
    skip_idempotence: bool,
) -> FormatterResult<()> {
    let content = read_input(input).map_err(|e| {
        FormatterError::Io(IoError::Filesystem(
            "Failed to read input contents".into(),
            e,
        ))
    })?;
    let query = read_input(query).map_err(|e| {
        FormatterError::Io(IoError::Filesystem(
            "Failed to read query contents".into(),
            e,
        ))
    })?;

    let mut configuration = Configuration::try_from(query.as_str())?;
    // Replace the language deduced from the query file by the one from the CLI, if any
    if let Some(l) = language {
        configuration.language = l
    }

    // All the work related to tree-sitter and the query is done here
    log::info!("Apply Tree-sitter query");
    let mut atoms = tree_sitter::apply_query(&content, &query, configuration.language)?;

    // Various post-processing of whitespace
    atoms.post_process();

    // Pretty-print atoms
    log::info!("Pretty-print output");
    let rendered = pretty::render(&atoms[..], configuration.indent_level)?;
    let trimmed = trim_whitespace(&rendered);

    if !skip_idempotence {
        idempotence_check(&trimmed, &query, language)?
    }

    write!(output, "{trimmed}")?;

    Ok(())
}

fn read_input(input: &mut dyn io::Read) -> Result<String, io::Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(content)
}

fn trim_whitespace(s: &str) -> String {
    // Trim whitespace from the end of each line,
    // then trim any leading/trailing new lines,
    // finally reinstate the new line at EOF.
    format!("{}\n", s.lines().map(str::trim_end).join("\n").trim())
}

fn idempotence_check(
    content: &str,
    query: &str,
    language: Option<Language>,
) -> FormatterResult<()> {
    log::info!("Checking for idempotence ...");

    let mut input = content.as_bytes();
    let mut query = query.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());
    let do_steps = || -> Result<(), FormatterError> {
        formatter(&mut input, &mut output, &mut query, language, true)?;
        let reformatted = String::from_utf8(output.into_inner()?)?;
        if content == reformatted {
            Ok(())
        } else {
            log::error!("Failed idempotence check");
            log::error!("{}", StrComparison::new(content, &reformatted));
            Err(FormatterError::Idempotence)
        }
    };
    let res = do_steps();
    if let Err(err) = res {
        match err {
            // If topiary ran smoothly on its own output,
            // but produced a different output, it is a Idempotence error.
            FormatterError::Idempotence => Err(FormatterError::Idempotence),
            // On the other hand, if it failed to run on its output,
            // it means that when formatting the code, topiary somehow broke it.
            // Hence it is a formatting error.
            _ => Err(FormatterError::Formatting(Box::new(err))),
        }
    } else {
        res
    }
}

#[test]
fn parse_error_fails_formatting() {
    let mut input = "[ 1, % ]".as_bytes();
    let mut output = Vec::new();
    let mut query = "(#language! json)".as_bytes();
    match formatter(&mut input, &mut output, &mut query, None, true) {
        Err(FormatterError::Parsing {
            start_line: 1,
            end_line: 1,
            ..
        }) => {}
        result => {
            panic!("Expected a parsing error on line 1, but got {result:?}");
        }
    }
}
