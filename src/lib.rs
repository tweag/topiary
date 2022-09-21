use clap::ArgEnum;
use error::{FormatterError, ReadingError};
use itertools::Itertools;
use pretty_assertions::assert_eq;
use std::{fs, io, path::Path};

mod error;
mod pretty;
mod syntax_info;
mod tree_sitter;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum Language {
    Json,
    Ocaml,
    Rust,
}

/// A Node from tree-sitter is turned into into a list of atoms
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Atom {
    Blankline,
    Empty,
    Hardline,
    IndentEnd,
    IndentStart,
    Leaf { content: String, id: usize },
    Literal(String),
    Softline { spaced: bool },
    Space,
}

pub type Result<T> = std::result::Result<T, FormatterError>;

pub fn formatter(
    input: &mut dyn io::Read,
    output: &mut dyn io::Write,
    language: Language,
    check_idempotence: bool,
) -> Result<()> {
    let content = read_input(input)?;
    let query = read_query(language)?;

    // All the work related to tree-sitter and the query is done here
    let query_result = tree_sitter::apply_query(&content, &query, language)?;
    let mut atoms = query_result.atoms;

    // Various post-processing of whitespace
    //
    // TODO: Make sure these aren't unnecessarily inefficient, in terms of
    // recreating a vector of atoms over and over.
    log::debug!("Before post-processing: {atoms:?}");
    put_before(&mut atoms, Atom::IndentEnd, Atom::Space, &[]);
    let mut atoms = trim_following(&atoms, Atom::Blankline, Atom::Space);
    put_before(&mut atoms, Atom::Hardline, Atom::Blankline, &[Atom::Space]);
    put_before(&mut atoms, Atom::IndentStart, Atom::Space, &[]);
    put_before(
        &mut atoms,
        Atom::IndentStart,
        Atom::Hardline,
        &[Atom::Space],
    );
    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[Atom::Space]);
    let atoms = trim_following(&atoms, Atom::Hardline, Atom::Space);
    let atoms = clean_up_consecutive(&atoms, Atom::Space);
    let mut atoms = clean_up_consecutive(&atoms, Atom::Hardline);
    ensure_final_hardline(&mut atoms);
    log::debug!("Final list of atoms: {atoms:?}");

    // Pretty-print atoms
    let rendered = pretty::render(&atoms, query_result.indent_level)?;
    let trimmed = trim_trailing_spaces(&rendered);

    if check_idempotence {
        idempotence_check(&trimmed, language)?
    }

    write!(output, "{trimmed}")?;

    Ok(())
}

fn read_input(input: &mut dyn io::Read) -> Result<String> {
    let mut content = String::new();
    input.read_to_string(&mut content).map_err(|e| {
        FormatterError::Reading(ReadingError::Io("Failed to read input content".into(), e))
    })?;
    Ok(content)
}

fn read_query(language: Language) -> Result<String> {
    let path = &str::to_lowercase(format!("languages/queries/{:?}.scm", language).as_str());
    let query = fs::read_to_string(Path::new(path)).map_err(|e| {
        FormatterError::Reading(ReadingError::Io(
            format!("Failed to read query file at {path}"),
            e,
        ))
    })?;
    Ok(query)
}

fn clean_up_consecutive(atoms: &[Atom], atom: Atom) -> Vec<Atom> {
    let filtered = atoms
        .split(|a| *a == atom)
        .filter(|chain| !chain.is_empty());

    Itertools::intersperse(filtered, &[atom.clone()])
        .flatten()
        .cloned()
        .collect_vec()
}

fn trim_following(atoms: &[Atom], delimiter: Atom, skip: Atom) -> Vec<Atom> {
    let trimmed = atoms
        .split(|a| *a == delimiter)
        .map(|slice| slice.iter().skip_while(|a| **a == skip).collect::<Vec<_>>());

    Itertools::intersperse(trimmed, vec![&delimiter])
        .flatten()
        .cloned()
        .collect_vec()
}

fn put_before(atoms: &mut Vec<Atom>, before: Atom, after: Atom, ignoring: &[Atom]) {
    for i in 0..atoms.len() - 1 {
        if atoms[i] == after {
            for j in i + 1..atoms.len() {
                if atoms[j] != before && atoms[j] != after && !ignoring.contains(&atoms[j]) {
                    // stop looking
                    break;
                }
                if atoms[j] == before {
                    // switch
                    atoms[i] = before.clone();
                    atoms[j] = after.clone();
                    break;
                }
            }
        }
    }
}

fn ensure_final_hardline(atoms: &mut Vec<Atom>) {
    if let Some(Atom::Hardline) = atoms.last() {
    } else {
        atoms.push(Atom::Hardline);
    }
}

fn trim_trailing_spaces(s: &str) -> String {
    Itertools::intersperse(s.split('\n').map(|line| line.trim_end()), "\n").collect::<String>()
}

fn idempotence_check(content: &str, language: Language) -> Result<()> {
    log::info!("Checking for idempotence ...");

    let mut input = content.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());
    formatter(&mut input, &mut output, language, false)?;
    let reformatted = String::from_utf8(output.into_inner()?)?;

    if content == reformatted {
        Ok(())
    } else {
        log::error!("Failed idempotence check");
        assert_eq!(content, reformatted);
        Err(FormatterError::Idempotence)
    }
}

#[test]
fn test_put_indent_ends_before_hardlines() {
    let mut atoms = vec![Atom::Hardline, Atom::Hardline, Atom::IndentEnd];
    let expected = vec![Atom::IndentEnd, Atom::Hardline, Atom::Hardline];
    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[]);
    assert_eq!(expected, atoms);
}

#[test]
fn test_put_indent_ends_before_hardlines_ignoring_space() {
    let mut atoms = vec![Atom::Hardline, Atom::Space, Atom::Hardline, Atom::IndentEnd];
    let expected = vec![Atom::IndentEnd, Atom::Space, Atom::Hardline, Atom::Hardline];

    put_before(&mut atoms, Atom::IndentEnd, Atom::Hardline, &[Atom::Space]);

    assert_eq!(expected, atoms);
}
