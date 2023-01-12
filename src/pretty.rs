use crate::{Atom, FormatterResult};
use pretty::RcDoc;

pub fn render(atoms: &[Atom], indent_level: isize) -> FormatterResult<String> {
    let doc = atoms_to_doc(&mut 0, atoms, indent_level);
    let mut rendered = String::new();
    doc.render_fmt(usize::max_value(), &mut rendered)?;
    Ok(rendered)
}

fn atoms_to_doc<'a>(i: &mut usize, atoms: &'a [Atom], indent_level: isize) -> RcDoc<'a, ()> {
    let mut doc = RcDoc::nil();

    while *i < atoms.len() {
        let atom = &atoms[*i];
        if let Atom::IndentEnd = atom {
            return doc;
        } else {
            doc = doc.append(match atom {
                Atom::Blankline => RcDoc::hardline().append(RcDoc::hardline()),
                Atom::Empty => RcDoc::text(""),
                &Atom::Hardline => RcDoc::hardline(),
                Atom::Leaf { content, .. } => RcDoc::text(content.trim_end()),
                Atom::Literal(s) => RcDoc::text(s),
                Atom::IndentEnd => unreachable!(),
                Atom::IndentStart => {
                    *i += 1;
                    atoms_to_doc(i, atoms, indent_level).nest(indent_level)
                }
                Atom::Softline { .. } => unreachable!(),
                Atom::Space => RcDoc::space(),
                Atom::DeleteBegin => unreachable!(),
                Atom::DeleteEnd => unreachable!(),
            });
        }
        *i += 1;
    }

    doc
}
