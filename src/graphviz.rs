/// GraphViz visualisation for our SyntaxTree representation
/// Named syntax nodes are elliptical; anonymous are rectangular
use std::{borrow::Cow, fmt, io};

use crate::{tree_sitter::SyntaxNode, FormatterResult};

/// We double-escape whitespace (\n and \t) so it is
/// rendered as the escaped value in the GraphViz output
fn escape(input: &str) -> Cow<str> {
    let mut buffer: Option<String> = None;

    let mut start: usize = 0;
    let length = input.len();

    let append = |buffer: &mut Option<String>, from: &mut usize, to: usize, suffix: &str| {
        // Allocate buffer only when necessary
        if buffer.is_none() {
            // Best case:  length + 1  (i.e., single escaped character in input)
            // Worst case: length * 3  (i.e., every character needs double-escaping)
            *buffer = Some(String::with_capacity(length * 2));
        }

        if let Some(ref mut buffer) = buffer {
            // Decant the unescaped chunk from the input,
            // followed by the escaped suffix provided
            *buffer += &input[*from..to];
            *buffer += suffix;
        }

        // Fast-forward the tracking cursor to the next character
        *from = to + 1;
    };

    for (idx, current) in input.chars().enumerate() {
        match current {
            // Double-escape whitespace characters
            '\n' => append(&mut buffer, &mut start, idx, r#"\\n"#),
            '\t' => append(&mut buffer, &mut start, idx, r#"\\t"#),

            otherwise => {
                // If char::escape_default starts with a backslash, then we
                // have an escaped character and we're off the happy path
                let mut escaped = otherwise.escape_default().peekable();
                if escaped.peek() == Some(&'\\') {
                    append(
                        &mut buffer,
                        &mut start,
                        idx,
                        &otherwise.escape_default().to_string(),
                    );
                }
            }
        }
    }

    if let Some(mut buffer) = buffer {
        // Decant whatever's left of the input into the buffer
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

#[cfg(test)]
mod test {
    use super::escape;

    #[test]
    fn double_escape() {
        // PBT would be handy, here
        assert_eq!(escape("foo"), "foo");
        assert_eq!(escape("'"), r#"\'"#);
        assert_eq!(escape("\n"), r#"\\n"#);
        assert_eq!(escape("\t"), r#"\\t"#);
        assert_eq!(
            escape("Here's something\nlonger"),
            r#"Here\'s something\\nlonger"#
        );
    }
}
