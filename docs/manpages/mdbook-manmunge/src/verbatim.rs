use std::borrow::Cow;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl From<pulldown_cmark_to_cmark::Error> for Error {
    fn from(value: pulldown_cmark_to_cmark::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

trait Escape {
    fn escape_backslashes(&self) -> Cow<str>;
}

impl Escape for String {
    fn escape_backslashes(&self) -> Cow<str> {
        // FIXME Only escape backslashes
        self.replace("\\", "\\\\").into()
    }
}

pub trait Verbatim<'parse> {
    /// Consume pulldown_cmark event
    fn consume(&mut self, event: Event<'parse>);

    /// Render consumed events as verbatim plain text
    fn render(&self) -> Result<String, Error>;

    /// Emit pulldown_cmark events within a code fence
    fn emit(&self) -> Result<Vec<Event<'parse>>, Error> {
        Ok(vec![
            // Opening code fence
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("".into()))),
            // Verbatim contents
            Event::Text(self.render()?.escape_backslashes().into_owned().into()),
            // Closing code fence
            Event::End(TagEnd::CodeBlock),
        ])
    }
}

/// Consume pulldown_cmark events and render them as Markdown
#[derive(Clone)]
pub struct Cmark<'parse> {
    events: Vec<Event<'parse>>,
}

impl<'parse> Cmark<'parse> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl<'parse> Verbatim<'parse> for Cmark<'parse> {
    fn consume(&mut self, event: Event<'parse>) {
        self.events.push(event);
    }

    fn render(&self) -> Result<String, Error> {
        let mut buf = String::new();

        pulldown_cmark_to_cmark::cmark_with_options(
            self.events.clone().into_iter(),
            &mut buf,
            pulldown_cmark_to_cmark::Options {
                increment_ordered_list_bullets: true,
                ..pulldown_cmark_to_cmark::Options::default()
            },
        )?;

        Ok(buf)
    }
}

// TODO Implement Table renderer as impl of Verbatim
