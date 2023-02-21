use std::str::FromStr;

use ouroboros::self_referencing;
use tree_sitter::{Parser, Query, QueryCapture, QueryCaptures, QueryCursor, QueryMatch, Tree};

use crate::{language::Language, FormatterError, FormatterResult};

/// Default indentation level (number of spaces)
static DEFAULT_INDENT_LEVEL: usize = 2;

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
    type Item = FormatterResult<Pragma<'a>>;
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
    type Item = FormatterResult<Pragma<'a>>;

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
                // NOTE It would be nice to implement this as TryFrom,
                // but I can't get the lifetime annotations right...

                // This should never happen... :P
                let parse_error = || {
                    FormatterError::Query("Could not parse pragma at node: {node:?}".into(), None)
                };

                // The predicate name is under the "name" field, which
                // consists of two sibling tokens: the "#" sigil and the
                // name itself.
                let predicate = (|| -> FormatterResult<&str> {
                    Ok(node
                        .child_by_field_name("name")
                        .ok_or_else(parse_error)?
                        .next_sibling()
                        .ok_or_else(parse_error)?
                        .utf8_text(fields.source)?)
                })();

                if let Err(error) = predicate {
                    return Some(Err(error));
                }

                // We take the entirety of the "parameters" field, which
                // can be post-processed if necessary.
                let value = (|| -> FormatterResult<Option<&str>> {
                    Ok(
                        match node
                            .child_by_field_name("parameters")
                            .ok_or_else(parse_error)?
                            .utf8_text(fields.source)?
                        {
                            // NOTE If the parsed value is an empty
                            // string, then there was no value.
                            "" => None,
                            value => Some(value),
                        },
                    )
                })();

                if let Err(error) = value {
                    return Some(Err(error));
                }

                return Some(Ok(Pragma {
                    predicate: predicate.unwrap(),
                    value: value.unwrap(),
                }));
            }

            // Stop iteration
            None
        })
    }
}

/// Language query configuration from parsed pragmata
pub struct Configuration {
    pub language: Language,
    pub indent_level: usize,
}

impl FromStr for Configuration {
    type Err = FormatterError;

    fn from_str(query: &str) -> FormatterResult<Self> {
        let mut language: Option<Language> = None;
        let mut indent_level: Option<usize> = None;

        let pragmata = Pragmata::from(query);
        for pragma in pragmata {
            let Pragma { predicate, value } = pragma?;
            log::info!("Pragma: {predicate}, Arguments: {value:?}");

            match predicate {
                "language" => {
                    if let Some(value) = value {
                        if language.is_some() {
                            log::warn!("The #language! pragma has already been set");
                        }

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
                        if indent_level.is_some() {
                            log::warn!("The #indent-level! pragma has already been set");
                        }

                        indent_level = Some(value.parse().map_err(|_| {
                            FormatterError::Query(
                                format!("The #indent-level! pragma expects a positive integer, but got '{value}'"),
                                None,
                            )
                        })?);
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
                indent_level: indent_level.unwrap_or(DEFAULT_INDENT_LEVEL),
            })
        } else {
            Err(FormatterError::Query(
                "The query file must set a language using the #language! pragma".into(),
                None,
            ))
        }
    }
}
