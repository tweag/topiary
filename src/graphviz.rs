/// GraphViz visualisation for our SyntaxTree representation
/// Named syntax nodes are elliptical; anonymous are rectangular
use std::{borrow::Cow, fmt, io};

use crate::{tree_sitter::SyntaxNode, FormatterResult};

// TODO Factor out the repetition here
// TODO Add tests for this
fn escape(input: &str) -> Cow<str> {
    // We double-escape whitespace (\n and \t) so it is
    // rendered as the escaped value in the GraphViz output
    let mut is_escaped = false;
    let mut buffer: String = String::new();
    let mut start: usize = 0;
    let length = input.len();

    for (upto, c) in input.chars().enumerate() {
        match c {
            '\n' => {
                if !is_escaped {
                    // Best case scenario: length + 1
                    // Worst case scenario: length * 3
                    buffer.reserve(length * 2);
                    is_escaped = true;
                }

                buffer += &input[start..upto];
                buffer += r#"\\n"#;

                start = upto + 1;
            }

            '\t' => {
                if !is_escaped {
                    buffer.reserve(length * 2);
                    is_escaped = true;
                }

                buffer += &input[start..upto];
                buffer += r#"\\t"#;

                start = upto + 1;
            }

            otherwise => {
                let mut escaped = otherwise.escape_default().peekable();
                if escaped.peek() == Some(&'\\') {
                    if !is_escaped {
                        buffer.reserve(length * 2);
                        is_escaped = true;
                    }

                    buffer += &input[start..upto];
                    buffer += &otherwise.escape_default().to_string();

                    start = upto + 1;
                }
            }
        }
    }

    if is_escaped {
        buffer += &input[start..length];
        buffer.into()
    } else {
        input.into()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shape = match self.is_named {
            true => "ellipse",
            false => "box",
        };

        writeln!(
            f,
            "  {} [label=\"{}\", shape={shape}];",
            self.id,
            escape(&self.kind)
        )?;

        for child in &self.children {
            writeln!(f, "  {} -- {};", self.id, child.id)?;
            write!(f, "{child}")?;
        }

        Ok(())
    }
}

pub fn write(output: &mut dyn io::Write, root: &SyntaxNode) -> FormatterResult<()> {
    writeln!(output, "graph {{")?;
    write!(output, "{root}")?;
    writeln!(output, "}}")?;

    Ok(())
}
