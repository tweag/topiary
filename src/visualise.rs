use std::io;

use serde::Serialize;

use crate::{configuration::Configuration, FormatterError, FormatterResult, Language};

#[derive(Clone, Copy, Debug)]
pub enum Visualisation {
    Json,
}

#[derive(Serialize)]
struct Position {
    row: usize,
    column: usize,
}

impl From<tree_sitter::Point> for Position {
    fn from(point: tree_sitter::Point) -> Self {
        Self {
            row: point.row + 1,
            column: point.column + 1,
        }
    }
}

#[derive(Serialize)]
struct Node {
    kind: String,
    is_named: bool,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    start: Position,
    end: Position,

    children: Vec<Node>,
}

impl Node {
    fn new(language: tree_sitter::Language, source: &str) -> FormatterResult<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language).map_err(|_| {
            FormatterError::Internal("Could not apply Tree-sitter grammar".into(), None)
        })?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| FormatterError::Internal("Could not parse input".into(), None))?;

        Ok(tree.root_node().into())
    }
}

impl From<tree_sitter::Node<'_>> for Node {
    fn from(node: tree_sitter::Node) -> Self {
        let mut walker = node.walk();
        let children = node.children(&mut walker).map(Node::from).collect();

        Self {
            children,

            kind: node.kind().into(),
            is_named: node.is_named(),
            is_extra: node.is_extra(),
            is_error: node.is_error(),
            is_missing: node.is_missing(),
            start: node.start_position().into(),
            end: node.end_position().into(),
        }
    }
}

pub fn visualiser(
    input: &mut dyn io::Read,
    output: &mut dyn io::Write,
    query: &mut dyn io::Read,
    language: Option<Language>,
    visualisation: Visualisation,
) -> FormatterResult<()> {
    // Read the input source
    let mut source = String::new();
    input.read_to_string(&mut source)?;

    // Set the language, where language > query file
    let language = match language {
        Some(language) => language,
        None => {
            // Read the query file to determine the language
            let mut buffer = String::new();
            query.read_to_string(&mut buffer)?;
            let configuration: Configuration = buffer.parse()?;

            configuration.language
        }
    };

    match visualisation {
        Visualisation::Json => {
            let tree = parse(&language.grammars(), &source)?;
            serde_json::to_writer(output, &tree)?;
        }
    }

    Ok(())
}

fn parse(grammars: &[tree_sitter::Language], source: &str) -> FormatterResult<Node> {
    grammars
        .iter()
        .map(|&grammar| Node::new(grammar, source))
        .fold(
            Err(FormatterError::Internal(
                "No grammar found for language".into(),
                None,
            )),
            Result::or,
        )
}
