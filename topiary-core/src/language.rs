use std::fmt;

use crate::TopiaryQuery;

/// Direction for processing grammar extras (inter-node content)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarExtrasDirection {
    /// Processing content to append after a node
    Append,
    /// Processing content to prepend before a node
    Prepend,
}

/// Trait for processing "grammar extras" (inter-node content like whitespace and comments)
/// in a language-specific way. This is used when captures like @append_grammar_extras
/// or @prepend_grammar_extras are encountered.
pub trait GrammarExtrasProcessor: fmt::Debug + Send + Sync {
    /// Process the gap content between nodes.
    ///
    /// # Arguments
    /// * `gap_content` - The raw bytes between two tree-sitter nodes
    /// * `direction` - Whether this gap will be appended or prepended
    /// * `context` - The full capture context with indent info and other state
    ///
    /// # Returns
    /// * `Some(Vec<Atom>)` - Use these atoms instead of default spacing
    ///   - Empty vec means strip all grammar extras in the gap
    ///   - Non-empty vec contains atoms to insert (e.g., Hardline, IndentStart, etc.)
    /// * `None` - Fall back to default spacing behavior
    fn process_gap(
        &self,
        gap_content: &[u8],
        direction: GrammarExtrasDirection,
        context: &crate::atom_collection::CaptureContext,
    ) -> Option<Vec<crate::Atom>>;
}

/// Default processor that doesn't modify grammar extras.
/// Always returns None to use default spacing rules.
#[derive(Debug)]
pub struct DefaultGrammarExtrasProcessor;

impl GrammarExtrasProcessor for DefaultGrammarExtrasProcessor {
    fn process_gap(
        &self,
        _gap_content: &[u8],
        _direction: GrammarExtrasDirection,
        _context: &crate::atom_collection::CaptureContext,
    ) -> Option<Vec<crate::Atom>> {
        None // Always use default spacing
    }
}

/// A Language contains all the information Topiary requires to format that
/// specific languages.
#[derive(Debug)]
pub struct Language {
    /// The name of the language, used as a key when looking up information in
    /// the Configuration, and to convert from a language to the respective tree-sitter
    /// grammar.
    pub name: String,
    /// The Query Topiary will use to get the formatting captures, must be
    /// present. The topiary engine does not include any formatting queries.
    pub query: TopiaryQuery,
    /// The tree-sitter Language. Topiary will use this Language for parsing.
    pub grammar: topiary_tree_sitter_facade::Language,
    /// The indentation string used for that particular language. Defaults to "  "
    /// if not provided. Any string can be provided, but in most instances will be
    /// some whitespace: "  ", "    ", or "\t".
    pub indent: Option<String>,
    /// Optional processor for handling grammar extras (inter-node content) in a
    /// language-specific way. Used with @append_grammar_extras and @prepend_grammar_extras.
    pub grammar_extras_processor: Option<Box<dyn GrammarExtrasProcessor>>,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Processor for bash and zsh line continuations.
///
/// Handles backslash-newline sequences by:
/// - Normalizing whitespace before the backslash to a single space
/// - Preserving the backslash-newline sequence
/// - Adding one level of indentation after the newline for continuation lines
#[derive(Debug)]
pub struct BashGrammarExtrasProcessor;

impl GrammarExtrasProcessor for BashGrammarExtrasProcessor {
    fn process_gap(
        &self,
        gap_content: &[u8],
        direction: GrammarExtrasDirection,
        context: &crate::atom_collection::CaptureContext,
    ) -> Option<Vec<crate::Atom>> {
        match direction {
            GrammarExtrasDirection::Append => {
                // For append: processing content after the current node (before next node)
                // Look for backslash-newline sequence
                if gap_content.windows(2).any(|w| w == b"\\\n") {
                    // Found a line continuation
                    // Return: space + backslash, then hardline (inherits current indent) + extra indent
                    Some(vec![
                        crate::Atom::Literal(" \\".to_string()),
                        crate::Atom::Hardline,
                        crate::Atom::Literal(context.indent.to_string()),
                    ])
                } else if gap_content.contains(&b'\n') {
                    // Found a regular newline (no backslash continuation)
                    // Preserve it as a hard line break with extra indent
                    Some(vec![
                        crate::Atom::Hardline,
                        crate::Atom::Literal(context.indent.to_string()),
                    ])
                } else {
                    // No line continuation or newline, use default spacing
                    None
                }
            }
            GrammarExtrasDirection::Prepend => {
                // For prepend: processing content before the current node (after prev node)
                // Look for backslash-newline sequence
                if gap_content.windows(2).any(|w| w == b"\\\n") {
                    // Found a line continuation before this node
                    // Return: space + backslash, then hardline (inherits current indent) + extra indent
                    Some(vec![
                        crate::Atom::Literal(" \\".to_string()),
                        crate::Atom::Hardline,
                        crate::Atom::Literal(context.indent.to_string()),
                    ])
                } else if gap_content.contains(&b'\n') {
                    // Found a regular newline
                    // Preserve it with extra indent
                    Some(vec![
                        crate::Atom::Hardline,
                        crate::Atom::Literal(context.indent.to_string()),
                    ])
                } else {
                    // No line continuation or newline, use default spacing
                    None
                }
            }
        }
    }
}
