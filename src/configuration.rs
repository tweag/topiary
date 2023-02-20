use crate::{language::Language, FormatterError, FormatterResult};
use std::str::FromStr;
use tree_sitter::{Parser, Query, QueryCursor};

/// Language pragmata are root-level predicates, which can be extracted with a simple query
static PRAGMA_QUERY: &str = r#"
    (program (predicate) @pragma)
"#;

#[derive(Debug)]
struct Pragma<'a> {
    predicate: &'a str,
    value: Option<&'a str>,
}

#[derive(Debug)]
struct Pragmata<'a> {
    source: &'a str,
}

impl<'a> From<&'a str> for Pragmata<'a> {
    fn from(source: &'a str) -> Self {
        Self { source }
    }
}

impl<'a> IntoIterator for Pragmata<'a> {
    type Item = &'a Pragma<'a>;
    type IntoIter = std::iter::Map; // FIXME

    fn into_iter(self) -> Self::IntoIter {
        let source = self.source.as_bytes();
        let language = tree_sitter_query::language();

        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        // Parse the source to find the root node
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();

        let query = Query::new(language, PRAGMA_QUERY).unwrap();
        let mut cursor = QueryCursor::new();

        cursor
            .captures(&query, root, source)
            .flat_map(|captures| captures.0.captures)
            .map(|capture| {
                // Convert the captured predicate node into a Pragma
                let node = capture.node;

                // The predicate name is under the "name" field, which
                // consists of two sibling tokens: the "#" sigil and
                // the name itself.
                let predicate = node
                    .child_by_field_name("name")
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .utf8_text(source)
                    .unwrap();

                // We take the entirety of the "parameters" field, which
                // can be post-processed if necessary. NOTE If the value
                // is an empty string, then there was no value.
                let value = match node
                    .child_by_field_name("parameters")
                    .unwrap()
                    .utf8_text(source)
                    .unwrap()
                {
                    "" => None,
                    value => Some(value),
                };

                Pragma { predicate, value }
            })
    }
}

pub struct Configuration {
    pub language: Language,
    pub indent_level: usize,
}

impl FromStr for Configuration {
    type Err = FormatterError;

    fn from_str(query: &str) -> FormatterResult<Self> {
        let mut language: Option<Language> = None;
        let mut indent_level: usize = 2;

        let pragmata = Pragmata::from(query);
        for Pragma { predicate, value } in pragmata {
            match predicate {
                &"language" => {
                    if let Some(value) = value {
                        language = Some(Language::new(value)?);
                    } else {
                        return Err(FormatterError::Query(
                            "The #language! pragma must have a parameter".into(),
                            None,
                        ));
                    }
                }

                &"indent-level" => {
                    if let Some(value) = value {
                        indent_level = value.parse().map_err(|_| {
                            FormatterError::Query(
                                format!("The #indent-level! pragma expects a positive integer, but got '{value}'"),
                                None,
                            )
                        })?;
                    } else {
                        return Err(FormatterError::Query(
                            "The #indent-level! pragma must have a parameter".into(),
                            None,
                        ));
                    }
                }

                _ => {
                    return Err(FormatterError::Query(
                        format!("Unknown pragma '#{predicate}!'"),
                        None,
                    ))
                }
            }
        }

        if let Some(language) = language {
            Ok(Configuration {
                language,
                indent_level,
            })
        } else {
            Err(FormatterError::Query(
                "The query file must set a language using the #language! pragma".into(),
                None,
            ))
        }
    }
}
