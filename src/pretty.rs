use std::fmt::Write;

use crate::{Atom, FormatterResult};

pub fn render(atoms: &[Atom], indent_offset: usize) -> FormatterResult<String> {
    let rendered = atoms_to_str(atoms, indent_offset);
    Ok(rendered)
}

fn atoms_to_str(
    atoms: &[Atom],
    indent_offset: usize,
) -> String {
    let mut buffer = String::new();
    let mut indent_level = 0;

    for atom in atoms {
        let extra = match atom {
            Atom::Blankline => format!("\n\n{}", " ".repeat(indent_level)),
            Atom::Empty => String::new(),
            Atom::Hardline => format!("\n{}", " ".repeat(indent_level)),
            Atom::IndentEnd => {
                indent_level -= indent_offset;
                String::new()
            },
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
            _ => unreachable!(),
        };
        write!(buffer, "{}", extra)?;
    }
    buffer
}
