use crate::{language::Language, FormatterError, FormatterResult};
use regex::Regex;
use unescape::unescape;

pub struct Configuration {
    pub language: Language,
    pub indent: String,
}

impl Configuration {
    pub fn parse(query: &str) -> FormatterResult<Self> {
        let mut language: Option<Language> = None;
        let mut indent: String = String::from("  ");

        // Match lines beginning with a predicate like this:
        // (#language! rust)
        // (#indent! "    ")
        // (#foo! 1 2 bar)
        let regex =
            Regex::new(r"(?m)^\(#(?P<predicate>.*?)!\s+(?P<arguments>.*?)\)").expect("valid regex");

        for capture in regex.captures_iter(query) {
            let predicate = capture
                .name("predicate")
                .expect("predicate capture group")
                .as_str();
            let argument = capture.name("arguments").map(|arg| arg.as_str());

            log::info!("Predicate: {predicate} -  Argument: {argument:?}");

            match predicate {
                "language" => {
                    if let Some(arg) = argument {
                        language = Some(Language::new(arg)?);
                    } else {
                        return Err(FormatterError::Query(
                            "The #language! configuration predicate must have a parameter".into(),
                            None,
                        ));
                    }
                }
                "indent" => {
                    if let Some(arg) = argument {
                        // Strip first and last " or ' from the string
                        // and unescape characters like \t
                        let arg = unescape(&arg[1..arg.len() - 1]).ok_or_else(|| {
                            FormatterError::Query(
                                format!(
                                    "The #indent! parameter could not be unescaped, got '{arg}'"
                                ),
                                None,
                            )
                        })?;

                        if arg.chars().all(char::is_whitespace) {
                            indent = arg;
                        } else {
                            return Err(FormatterError::Query(
                                format!(
                                    "The #indent! parameter must only contain whitespace, but got '{arg}'"
                                ),
                                None,
                            ));
                        };
                    } else {
                        return Err(FormatterError::Query(
                            "The #indent! configuration predicate must have a parameter".into(),
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
            Ok(Configuration { language, indent })
        } else {
            Err(FormatterError::Query("The query file must configure a language using the #language! configuration predicate".into(), None))
        }
    }
}
