use std::{cmp::Ord, fmt::Display};

use serde::Serialize;
use topiary_tree_sitter_facade::{InputEdit, Node, Parser, Point, Tree};

use crate::{error::FormatterError, FormatterResult};

/// A module for common, low-level types and functions in the topiary-core crate

/// Refers to a position within the code. Used for error reporting, and for
/// comparing input with formatted output. The numbers are 1-based, because that
/// is how editors usually refer to a position. Derived from tree_sitter::Point.
/// Note that the order is the standard western reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Position {
    pub row: u32,
    pub column: u32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "({},{})", self.row, self.column)
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            row: point.row() + 1,
            column: point.column() + 1,
        }
    }
}

/// Some section of contiguous characters in the input.
/// It is assumed that `start <= end`, according to the order on `Position`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InputSection {
    pub start: Position,
    pub end: Position,
}

impl InputSection {
    pub fn contains(self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

impl From<&Node<'_>> for InputSection {
    fn from(value: &Node) -> Self {
        InputSection {
            start: value.start_position().into(),
            end: value.end_position().into(),
        }
    }
}

impl From<&InputEdit> for InputSection {
    fn from(value: &InputEdit) -> Self {
        InputSection {
            start: value.start_position().into(),
            end: value.old_end_position().into(),
        }
    }
}

/// A generic trait to subtract stuff from other stuff. The function can be partial.
/// In practice, it will be used to update text positions within the input,
/// when removing parts of it.
pub trait Diff<T> {
    type ErrorType;

    fn subtract(&mut self, other: T) -> Result<(), Self::ErrorType>;
}

/// Parses some string into a syntax tree, given a tree-sitter grammar.
pub fn parse(
    content: &str,
    grammar: &topiary_tree_sitter_facade::Language,
    tolerate_parsing_errors: bool,
    old_tree: Option<&Tree>,
) -> FormatterResult<Tree> {
    let mut parser = Parser::new()?;
    parser.set_language(grammar).map_err(|_| {
        FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
    })?;

    let tree = parser
        .parse(content, old_tree)?
        .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;

    // Fail parsing if we don't get a complete syntax tree.
    if !tolerate_parsing_errors {
        check_for_error_nodes(&tree.root_node())?;
    }

    Ok(tree)
}

fn check_for_error_nodes(node: &Node) -> FormatterResult<()> {
    if node.kind() == "ERROR" {
        let start = node.start_position();
        let end = node.end_position();

        // Report 1-based lines and columns.
        return Err(FormatterError::Parsing {
            start_line: start.row() + 1,
            start_column: start.column() + 1,
            end_line: end.row() + 1,
            end_column: end.column() + 1,
        });
    }

    for child in node.children(&mut node.walk()) {
        check_for_error_nodes(&child)?;
    }

    Ok(())
}
