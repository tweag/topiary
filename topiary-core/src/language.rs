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
    /// * `indent` - The indent string configured for this language (e.g., "  " or "\t")
    ///
    /// # Returns
    /// * `Some(String)` - Use this processed string instead of default spacing
    ///   - Empty string ("") means strip all grammar extras in the gap
    ///   - Non-empty string is the exact content to insert (including any indent)
    /// * `None` - Fall back to default spacing behavior
    fn process_gap(
        &self,
        gap_content: &[u8],
        direction: GrammarExtrasDirection,
        indent: &str,
    ) -> Option<String>;
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
        _indent: &str,
    ) -> Option<String> {
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
/// - Adding configured indentation after the newline
#[derive(Debug)]
pub struct BashGrammarExtrasProcessor;

impl GrammarExtrasProcessor for BashGrammarExtrasProcessor {
    fn process_gap(
        &self,
        gap_content: &[u8],
        _direction: GrammarExtrasDirection,
        indent: &str,
    ) -> Option<String> {
        // Look for backslash-newline sequence
        if gap_content.windows(2).any(|w| w == b"\\\n") {
            // Found a line continuation
            // Return: space + backslash + newline + configured indent
            Some(format!(" \\\n{}", indent))
        } else {
            // No line continuation, use default spacing
            None
        }
    }
}
