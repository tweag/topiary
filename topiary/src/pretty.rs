use std::fmt::Write;

use crate::{Atom, FormatterError, FormatterResult};

pub fn render(atoms: &[Atom], indent: &str) -> FormatterResult<String> {
    let mut buffer = String::new();
    let mut indent_level: usize = 0;

    for atom in atoms {
        match atom {
            Atom::Blankline => write!(buffer, "\n\n{}", indent.repeat(indent_level))?,

            Atom::Empty => (),

            Atom::Hardline => write!(buffer, "\n{}", indent.repeat(indent_level))?,

            Atom::IndentEnd => {
                if indent_level == 0 {
                    return Err(FormatterError::Query(
                        "Trying to close an unopened indentation block".into(),
                        None,
                    ));
                }

                indent_level -= 1;
            }

            Atom::IndentStart => indent_level += 1,

            Atom::Leaf {
                content,
                single_line_no_indent,
                ..
            } => {
                if *single_line_no_indent {
                    // The line break after the content has been previously added
                    // as a `Hardline` in the atom stream.
                    writeln!(buffer)?;
                }
                write!(buffer, "{}", content.trim_end())?
            }

            Atom::Literal(s) => write!(buffer, "{s}")?,

            Atom::Space => write!(buffer, " ")?,

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
    }

    Ok(buffer)
}
