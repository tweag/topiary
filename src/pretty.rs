use std::fmt::Write;

use crate::{Atom, FormatterError, FormatterResult};

pub fn render(atoms: &[Atom], indent_offset: usize) -> FormatterResult<String> {
    let mut buffer = String::new();
    let mut indent_level = 0;

    for atom in atoms {
        let extra = match atom {
            Atom::Blankline => format!("\n\n{}", " ".repeat(indent_level)),
            Atom::Empty => String::new(),
            Atom::Hardline => format!("\n{}", " ".repeat(indent_level)),
            Atom::IndentEnd => {
                if indent_offset > indent_level {
                    return Err(FormatterError::Query(
                        "Trying to close an unopened indentation block".into(),
                        None,
                    ));
                } else {
                    indent_level -= indent_offset;
                    String::new()
                }
            }
            Atom::IndentStart => {
                indent_level += indent_offset;
                String::new()
            }
            Atom::Leaf {
                content,
                single_line_no_indent,
                ..
            } => {
                if *single_line_no_indent {
                    // The line break after the content has been previously added
                    // as a `Hardline` in the atom stream.
                    format!("\n{}", content.trim_end())
                } else {
                    content.trim_end().to_string()
                }
            }
            Atom::Literal(s) => s.to_string(),
            Atom::Space => " ".to_string(),
            // All other atom kinds should have been post-processed at that point
            other => {
                return Err(FormatterError::Internal(
                    format!(
                        "Found atom that should have been removed before rendering: {:?}",
                        other
                    ),
                    None,
                ))
            }
        };
        write!(buffer, "{}", extra)?;
    }
    Ok(buffer)
}
