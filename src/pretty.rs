use crate::{Atom, FormatterResult};
use core::iter::repeat;
use pretty::RcDoc;

pub fn render(atoms: &[Atom], indent_offset: usize) -> FormatterResult<String> {
    let doc = atoms_to_doc(&mut 0, atoms, indent_offset, &mut 0);
    let mut rendered = String::new();
    doc.render_fmt(usize::max_value(), &mut rendered)?;
    Ok(rendered)
}

fn atoms_to_doc<'a>(
    i: &mut usize,
    atoms: &'a [Atom],
    indent_offset: usize,
    indent_level: &mut usize,
) -> RcDoc<'a, ()> {
    let mut doc = RcDoc::nil();

    while *i < atoms.len() {
        let atom = &atoms[*i];
        if let Atom::IndentEnd = atom {
            return doc;
        } else {
            doc = doc.append(match atom {
                Atom::Blankline => RcDoc::hardline()
                    .append(RcDoc::hardline())
                    .append(RcDoc::concat(repeat(RcDoc::space()).take(*indent_level))),
                Atom::Empty => RcDoc::text(""),
                &Atom::Hardline => RcDoc::hardline()
                    .append(RcDoc::concat(repeat(RcDoc::space()).take(*indent_level))),
                Atom::Leaf { content, .. } => RcDoc::text(content.trim_end()),
                Atom::Literal(s) => RcDoc::text(s),
                Atom::MultilineOnlyLiteral { .. } => unreachable!(),
                Atom::IndentEnd => unreachable!(),
                Atom::IndentStart => {
                    *i += 1;
                    *indent_level += indent_offset;
                    let res = atoms_to_doc(i, atoms, indent_offset, indent_level);
                    *indent_level -= indent_offset;
                    res
                }
                Atom::Softline { .. } => unreachable!(),
                Atom::Space => RcDoc::space(),
                Atom::DeleteBegin => unreachable!(),
                Atom::DeleteEnd => unreachable!(),
                Atom::ScopedSoftline { .. } => unreachable!(),
            });
        }
        *i += 1;
    }

    doc
}
