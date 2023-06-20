//! After being split into Atoms, and the queries having been applied this
//! module is reponsible for rendering the slice of Atoms back into a displayable
//! format.

use std::{borrow::Cow, fmt::Write};

use log::warn;
use regex::Regex;

use crate::{Atom, FormatterError, FormatterResult};

/// Renders a slice of Atoms into an owned string.
/// The indent &str is used when an `Atom::IdentStart` is encountered.
/// Any string is accepted, but you will probably want to specify something
/// along the lines of "  " "    " or "\t".
///
/// # Errors
///
/// If an unexpected Atom is encountered, a `FormatterError::Internal` is returned.
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
                multi_line_indent_all,
                ..
            } => {
                if *single_line_no_indent {
                    // The line break after the content has been previously added
                    // as a `Hardline` in the atom stream.
                    writeln!(buffer)?;
                }

                let content = content.trim_end_matches('\n');

                let content = if *multi_line_indent_all {
                    warn!("before replacement: {:?}", content);
                    // Look for beginning of lines which may or may not be followed by the right
                    // amount of indenting. We'll replace that with a newline plus indenting.
                    let leaf_indent_regex =
                        Regex::new(&format!("\n({})?", indent.repeat(indent_level))).unwrap();

                    let replaced = leaf_indent_regex
                        .replace_all(
                            content, //.trim_end_matches('\n')
                            //.replace('\n', &("\n".to_string() + &(indent.repeat(indent_level))))
                            //.replace('\n', &("\nX".to_string()))
                            "\n".to_string() + &indent.repeat(indent_level),
                        )
                        .clone();
                    let replaced2: Cow<'_, str> =
                        Cow::Owned(replaced.trim_end_matches('\n').clone().into());

                    warn!("replaced: {:?}", replaced2);

                    replaced2.clone()
                } else {
                    content.into()
                };

                write!(buffer, "{}", content)?;
            }

            Atom::Literal(s) => write!(buffer, "{s}")?,

            Atom::Space => write!(buffer, " ")?,

            // All other atom kinds should have been post-processed at that point
            other => {
                return Err(FormatterError::Internal(
                    format!("Found atom that should have been removed before rendering: {other:?}",),
                    None,
                ))
            }
        };
    }

    Ok(buffer)
}
