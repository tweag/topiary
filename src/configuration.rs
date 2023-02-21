use std::str::FromStr;

use ouroboros::self_referencing;
use tree_sitter::{Parser, Query, QueryCapture, QueryCaptures, QueryCursor, QueryMatch, Tree};

use crate::{language::Language, FormatterError, FormatterResult};

/// Language pragmata are root-level predicates,
/// which can be extracted with a simple Tree-Sitter query
static PRAGMA_QUERY: &str = r#"
    (program (predicate) @pragma)
"#;

struct Pragma<'a> {
    predicate: &'a str,
    value: Option<&'a str>,
}

struct Pragmata<'a> {
    source: &'a [u8],
}

#[self_referencing]
struct PragmataIter<'a> {
    source: &'a [u8],
    tree: Tree,
    cursor: QueryCursor,
    query: Query,

    #[borrows(mut cursor, query, tree, source)]
    #[covariant]
    captures: QueryCaptures<'this, 'this, &'this [u8]>,
}

impl<'a> From<&'a str> for Pragmata<'a> {
    fn from(source: &'a str) -> Self {
        let source = source.as_bytes();
        Self { source }
    }
}

impl<'a> IntoIterator for Pragmata<'a> {
    type Item = Pragma<'a>;
    type IntoIter = PragmataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let source = self.source;
        let language = tree_sitter_query::language();

        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        let tree = parser.parse(source, None).unwrap();

        let query = Query::new(language, PRAGMA_QUERY).unwrap();
        let cursor = QueryCursor::new();

        PragmataIterBuilder {
            source,
            tree,
            cursor,
            query,

            captures_builder: |cursor, query, tree, source| {
                cursor.captures(query, tree.root_node(), source)
            },
        }
        .build()
    }
}

impl<'a> Iterator for PragmataIter<'a> {
    type Item = Pragma<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_mut(|fields| {
            if let Some((
                QueryMatch {
                    // The query will ensure there is exactly one
                    // capture in the match; of which, we only care
                    // about the node
                    captures: [QueryCapture { node, .. }, ..],
                    ..
                },
                _,
            )) = fields.captures.next()
            {
                // Convert the captured predicate node into a Pragma
                // NOTE We can safely unwrap _most_ of the time...

                // The predicate name is under the "name" field, which
                // consists of two sibling tokens: the "#" sigil and the
                // name itself.
                let predicate = node
                    .child_by_field_name("name")
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .utf8_text(fields.source)
                    .unwrap();

                // We take the entirety of the "parameters" field, which
                // can be post-processed if necessary. NOTE If the value
                // is an empty string, then there was no value.
                let value = match node
                    .child_by_field_name("parameters")
                    .unwrap()
                    .utf8_text(fields.source)
                    .unwrap()
                {
                    "" => None,
                    value => Some(value),
                };

                return Some(Pragma { predicate, value });
            }

            None
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
                "language" => {
                    if let Some(value) = value {
                        language = Some(Language::new(value)?);
                    } else {
                        return Err(FormatterError::Query(
                            "The #language! pragma must have a parameter".into(),
                            None,
                        ));
                    }
                }

                "indent-level" => {
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
