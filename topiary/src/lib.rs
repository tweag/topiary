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

use std::io;

use itertools::Itertools;
use pretty_assertions::StrComparison;

pub use crate::{
    configuration::Configuration,
    error::{FormatterError, IoError},
    language::{Language, SupportedLanguage},
    tree_sitter::{apply_query, SyntaxNode, Visualisation},
};

mod atom_collection;
mod configuration;
mod error;
mod graphviz;
mod language;
mod pretty;
mod tree_sitter;

/// An atom represents a small piece of the output. We turn Tree-sitter nodes
/// into atoms, and we add white-space atoms where appropriate. The final list
/// of atoms is rendered to the output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Atom {
    /// We don't allow consecutive `Hardline`, but a `Blankline` will render two
    /// newlines to produce a blank line.
    Blankline,
    /// A "no-op" atom that will not produce any output.
    #[default]
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
    /// Represents a softline. It will be turned into a hardline for multi-line
    /// constructs, and either a space or nothing for single-line constructs.
    Softline {
        spaced: bool,
    },
    /// Represents a space. Consecutive spaces are reduced to one before rendering.
    Space,
    /// Represents the destruction of errant spaces. Adjacent consecutive spaces are
    /// reduced to zero before rendering.
    Antispace,
    /// Represents a segment to be deleted.
    // It is a segment, because if one wants to delete a node,
    // it might happen that it contains several leaves.
    DeleteBegin,
    DeleteEnd,
    /// Scoped commands
    // ScopedSoftline works together with the @begin_scope and @end_scope query tags.
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
    // Represents an atom that must only be output if the associated scope meets the condition
    // (single-line or multi-line)
    ScopedConditional {
        id: usize,
        scope_id: String,
        condition: ScopeCondition,
        atom: Box<Atom>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeCondition {
    SingleLineOnly,
    MultiLineOnly,
}

/// A convenience wrapper around `std::result::Result<T, FormatterError>`.
pub type FormatterResult<T> = std::result::Result<T, FormatterError>;

/// Operations that can be performed by the formatter.
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Format { skip_idempotence: bool },
    Visualise { output_format: Visualisation },
}

/// The function that takes an input and formats, or visualises an output.
///
/// # Errors
///
/// If formatting fails for any reason, a `FormatterError` will be returned.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// use std::fs::File;
/// use std::io::{BufReader, Read};
/// use topiary::{formatter, Configuration, FormatterError, Operation};
///
/// let input = "[1,2]".to_string();
/// let mut input = input.as_bytes();
/// let mut output = Vec::new();
/// let mut query_file = BufReader::new(File::open("../languages/json.scm").expect("query file"));
/// let mut query = String::new();
/// query_file.read_to_string(&mut query).expect("read query file");
///
/// let config = Configuration::parse_default_config();
/// let language = config.get_language("json").unwrap();
/// let grammars = language
///     .grammars()
///     .await
///     .expect("grammars");
///
/// match formatter(&mut input, &mut output, &query, &language, &grammars, Operation::Format{ skip_idempotence: false }) {
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
/// # }) // end tokio_test
/// ```
pub fn formatter(
    input: &mut impl io::Read,
    output: &mut impl io::Write,
    query: &str,
    language: &Language,
    grammars: &[tree_sitter_facade::Language],
    operation: Operation,
) -> FormatterResult<()> {
    let content = read_input(input).map_err(|e| {
        FormatterError::Io(IoError::Filesystem(
            "Failed to read input contents".into(),
            e,
        ))
    })?;

    match operation {
        Operation::Format { skip_idempotence } => {
            // All the work related to tree-sitter and the query is done here
            log::info!("Apply Tree-sitter query");
            let mut atoms = tree_sitter::apply_query(&content, query, grammars, false)?;

            // Various post-processing of whitespace
            atoms.post_process();

            // Pretty-print atoms
            log::info!("Pretty-print output");
            let rendered = pretty::render(
                &atoms[..],
                // Default to "  " is the language has no indentation specified
                language.indent.as_ref().map_or("  ", |v| v.as_str()),
            )?;
            let trimmed = trim_whitespace(&rendered);

            if !skip_idempotence {
                idempotence_check(&trimmed, query, language, grammars)?;
            }

            write!(output, "{trimmed}")?;
        }

        Operation::Visualise { output_format } => {
            let (tree, _) = tree_sitter::parse(&content, grammars)?;
            let root: SyntaxNode = tree.root_node().into();

            match output_format {
                Visualisation::GraphViz => graphviz::write(output, &root)?,
                Visualisation::Json => serde_json::to_writer(output, &root)?,
            };
        }
    };

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
    language: &Language,
    grammars: &[tree_sitter_facade::Language],
) -> FormatterResult<()> {
    log::info!("Checking for idempotence ...");

    let mut input = content.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    formatter(
        &mut input,
        &mut output,
        query,
        language,
        grammars,
        Operation::Format {
            skip_idempotence: true,
        },
    )?;
    let reformatted = String::from_utf8(output.into_inner()?)?;
    let res = if content == reformatted {
        Ok(())
    } else {
        log::error!("Failed idempotence check");
        log::error!("{}", StrComparison::new(content, &reformatted));
        Err(FormatterError::Idempotence)
    };

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

#[tokio::test]
async fn parse_error_fails_formatting() {
    let mut input = "[ 1, % ]".as_bytes();
    let mut output = Vec::new();
    let query = "(#language! json)";
    let configuration = Configuration::parse_default_config();
    let language = configuration.get_language("json").unwrap();
    let grammars = language.grammars().await.unwrap();

    match formatter(
        &mut input,
        &mut output,
        query,
        language,
        &grammars,
        Operation::Format {
            skip_idempotence: true,
        },
    ) {
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
