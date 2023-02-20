use crate::{language::Language, FormatterError, FormatterResult};
use std::str::FromStr;

struct Directive<'a> {
    predicate: Value<'a>,
    value: Value<'a>,
}

enum Value<'a> {
    Numeric(usize),
    Symbol(&'a str),
    Text(String),
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

        todo!()
    }
}

/*
impl Configuration {
    pub fn parse(query: &str) -> FormatterResult<Self> {
        let mut language: Option<Language> = None;
        let mut indent_level: usize = 2;

        // Match lines beginning with a predicate like this:
        // (#language! rust)
        // (#indent-level! 4)
        // (#foo! 1 2 bar)
        let regex =
            Regex::new(r"(?m)^\(#(?P<predicate>.*?)!\s+(?P<arguments>.*?)\)").expect("valid regex");

        for capture in regex.captures_iter(query) {
            let predicate = capture
                .name("predicate")
                .expect("predicate capture group")
                .as_str();
            let mut arguments = capture
                .name("arguments")
                .expect("arguments capture group")
                .as_str()
                .split(' ');
            log::info!("Predicate: {predicate} -  Arguments: {arguments:?}");

            match predicate {
                "language" => {
                    if let Some(arg) = arguments.next() {
                        language = Some(Language::new(arg)?);
                    } else {
                        return Err(FormatterError::Query(
                            "The #language! configuration predicate must have a parameter".into(),
                            None,
                        ));
                    }
                }
                "indent-level" => {
                    if let Some(arg) = arguments.next() {
                        indent_level = arg.parse::<usize>().map_err(|_| {
                            FormatterError::Query(
                                format!(
                                    "The #indent-level! parameter must be a positive integer, but got '{arg}'"
                                ),
                                None,
                            )
                        })?;
                    } else {
                        return Err(FormatterError::Query(
                            "The #indent-level! configuration predicate must have a parameter"
                                .into(),
                            None,
                        ));
                    }
                }
                _ => {
                    return Err(FormatterError::Query(
                        format!("Unknown configuration predicate '{predicate}'"),
                        None,
                    ))
                }
            };
        }

        if let Some(language) = language {
            Ok(Configuration {
                language,
                indent_level,
            })
        } else {
            Err(FormatterError::Query("The query file must configure a language using the #language! configuration predicate".into(), None))
        }
    }
}
*/
