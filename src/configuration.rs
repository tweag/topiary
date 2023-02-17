/// We need a "quick-and-dirty" grammar to parse the query language and extract the configuration
/// predicates that exist only at the root level, throwing away everything else.
use crate::{language::Language, FormatterError, FormatterResult};

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{alpha1, anychar, char, digit1, one_of, space1},
    combinator::map_res,
    sequence::{delimited, preceded},
    IResult,
};

struct Directive<'a> {
    predicate: Value<'a>,
    value: Value<'a>,
}

enum Value<'a> {
    Numeric(usize),
    Symbol(&'a str),
    Text(String),
}

/// ROOT := (WS* [DIRECTIVE RULE COMMENT])* WS*
fn root(input: &str) -> IResult<&str, Vec<Directive>> {
    todo!()
}

/// DIRECTIVE := "(#" IDENTIFIER "!" WS+ (IDENTIFIER | STRING | NUMBER) ")"
fn directive(input: &str) -> IResult<&str, Option<Directive>> {
    let (remaining, _) = tag("(#")(input)?;
    let (remaining, predicate) = identifier(remaining)?;
    let (remaining, value) = preceded(space1, alt((identifier, string, number)))(remaining)?;
    let (remaining, _) = tag(")")(remaining)?;

    Ok((remaining, Some(Directive { predicate, value })))
}

/// IDENTIFIER := ALPHA ["-" "_" ALPHA]*
fn identifier(input: &str) -> IResult<&str, Value> {
    todo!();
}

/// STRING := "\"" ANY+ "\""
fn string(input: &str) -> IResult<&str, Value> {
    // FIXME This needs work
    let (remaining, value) = delimited(
        char('\"'),
        escaped(anychar, '\\', one_of(r#""nt\"#)),
        char('\"'),
    )(input)?;

    Ok((remaining, Value::Text(value.into())))
}

/// NUMBER := DIGIT+
fn number(input: &str) -> IResult<&str, Value> {
    let (remaining, value) = map_res(digit1, str::parse)(input)?;
    Ok((remaining, Value::Numeric(value)))
}

/// RULE := (ALTERNATION | CAPTURE) (WS+ TAG)*
/// COMMENT := ";" ANY* LF
/// ALTERNATION := "[" ANY+ "]"
/// CAPTURE := "(" ANY+ ")"
/// TAG := "@" IDENTIFIER

pub struct Configuration {
    pub language: Language,
    pub indent_level: usize,
}

impl TryFrom<&str> for Configuration {
    type Error = FormatterError;

    fn try_from(query: &str) -> FormatterResult<Self> {
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
