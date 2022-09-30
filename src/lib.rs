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
//! [GitHub](https://github.com/tweag/tree-sitter-formatter).

use configuration::Configuration;
pub use error::{FormatterError, ReadingError, WritingError};
use itertools::Itertools;
pub use language::Language;
use pretty_assertions::assert_eq;
use std::io;

mod configuration;
mod error;
mod language;
mod pretty;
mod syntax_info;
mod tree_sitter;

/// An atom represents a small piece of the output. We turn Tree-sitter nodes
/// into atoms, and we add white-space atoms where appropriate. The final list
/// of atoms is rendered to the output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Atom {
    /// We don't allow consecutive `Hardline`, but if a `Hardline` is followed by
    /// a `Blankline` we will render two newlines to produce a blank line.
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
    Leaf { content: String, id: usize },
    /// Represents a literal string, such as a semicolon.
    Literal(String),
    /// Represents a softline. It will be turned into a hardline for multi-line
    /// constructs, and either a space or nothing for single-line constructs.
    Softline { spaced: bool },
    /// Represents a space. Consecutive spaces are reduced to one before rendering.
    Space,
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
/// use tree_sitter_formatter::{formatter, FormatterError};
///
/// let input = "[1,2]".to_string();
/// let mut input = input.as_bytes();
/// let mut output = Vec::new();
/// let mut query = BufReader::new(File::open("languages/json.scm").expect("query file"));
///
/// match formatter(&mut input, &mut output, &mut query, false) {
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
    skip_idempotence: bool,
) -> FormatterResult<()> {
    let content = read_input(input).map_err(|e| {
        FormatterError::Reading(ReadingError::Io("Failed to read input content".into(), e))
    })?;
    let query = read_input(query).map_err(|e| {
        FormatterError::Reading(ReadingError::Io("Failed to read query content".into(), e))
    })?;

    let configuration = Configuration::parse(&query)?;

    // All the work related to tree-sitter and the query is done here
    let query_result = tree_sitter::apply_query(&content, &query, configuration.language)?;
    let mut atoms = query_result.atoms;

    // Various post-processing of whitespace
    //
    // TODO: Make sure these aren't unnecessarily inefficient, in terms of
    // recreating a vector of atoms over and over.
    log::debug!("Before post-processing: {atoms:?}");
    put_before(&mut atoms, Atom::IndentEnd, Atom::Space, &[]);
    let mut atoms = trim_following(&atoms, Atom::Blankline, Atom::Space);
    put_before(&mut atoms, Atom::Hardline, Atom::Blankline, &[Atom::Space]);
    put_before(&mut atoms, Atom::IndentStart, Atom::Space, &[]);
    put_before(
        &mut atoms,
        Atom::IndentStart,
        Atom::Hardline,
        &[Atom::Space],
    );
    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[Atom::Space]);
    let atoms = trim_following(&atoms, Atom::Hardline, Atom::Space);
    let atoms = clean_up_consecutive(&atoms, Atom::Space);
    let mut atoms = clean_up_consecutive(&atoms, Atom::Hardline);
    ensure_final_hardline(&mut atoms);
    log::debug!("Final list of atoms: {atoms:?}");

    // Pretty-print atoms
    let rendered = pretty::render(&atoms, query_result.indent_level)?;
    let trimmed = trim_trailing_spaces(&rendered);

    if !skip_idempotence {
        idempotence_check(&trimmed, &query)?
    }

    write!(output, "{trimmed}")?;

    Ok(())
}

fn read_input(input: &mut dyn io::Read) -> Result<String, io::Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(content)
}

fn clean_up_consecutive(atoms: &[Atom], atom: Atom) -> Vec<Atom> {
    let filtered = atoms
        .split(|a| *a == atom)
        .filter(|chain| !chain.is_empty());

    Itertools::intersperse(filtered, &[atom.clone()])
        .flatten()
        .cloned()
        .collect_vec()
}

fn trim_following(atoms: &[Atom], delimiter: Atom, skip: Atom) -> Vec<Atom> {
    let trimmed = atoms
        .split(|a| *a == delimiter)
        .map(|slice| slice.iter().skip_while(|a| **a == skip).collect::<Vec<_>>());

    Itertools::intersperse(trimmed, vec![&delimiter])
        .flatten()
        .cloned()
        .collect_vec()
}

fn put_before(atoms: &mut Vec<Atom>, before: Atom, after: Atom, ignoring: &[Atom]) {
    for i in 0..atoms.len() - 1 {
        if atoms[i] == after {
            for j in i + 1..atoms.len() {
                if atoms[j] != before && atoms[j] != after && !ignoring.contains(&atoms[j]) {
                    // stop looking
                    break;
                }
                if atoms[j] == before {
                    // switch
                    atoms[i] = before.clone();
                    atoms[j] = after.clone();
                    break;
                }
            }
        }
    }
}

fn ensure_final_hardline(atoms: &mut Vec<Atom>) {
    if let Some(Atom::Hardline) = atoms.last() {
    } else {
        atoms.push(Atom::Hardline);
    }
}

fn trim_trailing_spaces(s: &str) -> String {
    Itertools::intersperse(s.split('\n').map(|line| line.trim_end()), "\n").collect::<String>()
}

fn idempotence_check(content: &str, query: &str) -> FormatterResult<()> {
    log::info!("Checking for idempotence ...");

    let mut input = content.as_bytes();
    let mut query = query.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());
    formatter(&mut input, &mut output, &mut query, true)?;
    let reformatted = String::from_utf8(output.into_inner()?)?;

    if content == reformatted {
        Ok(())
    } else {
        log::error!("Failed idempotence check");
        assert_eq!(content, reformatted);
        Err(FormatterError::Idempotence)
    }
}

#[test]
fn test_put_indent_ends_before_hardlines() {
    let mut atoms = vec![Atom::Hardline, Atom::Hardline, Atom::IndentEnd];
    let expected = vec![Atom::IndentEnd, Atom::Hardline, Atom::Hardline];
    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[]);
    assert_eq!(expected, atoms);
}

#[test]
fn test_put_indent_ends_before_hardlines_ignoring_space() {
    let mut atoms = vec![Atom::Hardline, Atom::Space, Atom::Hardline, Atom::IndentEnd];
    let expected = vec![Atom::IndentEnd, Atom::Space, Atom::Hardline, Atom::Hardline];

    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[Atom::Space]);

    assert_eq!(expected, atoms);
}

#[test]
fn parse_error_fails_formatting() {
    let mut input = "[ 1, % ]".as_bytes();
    let mut output = Vec::new();
    let mut query = "(#language! json)".as_bytes();
    match formatter(&mut input, &mut output, &mut query, true) {
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
