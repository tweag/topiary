use std::borrow::Cow;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

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
    type ErrorType;

    /// Constructor
    fn new() -> Self;

    /// Consume pulldown_cmark event
    fn consume(&mut self, event: Event<'parse>);

    /// Render consumed events as verbatim plain text
    fn render(&self) -> Result<String, Self::ErrorType>;

    /// Emit pulldown_cmark events within a code fence
    fn emit(&self) -> Result<Vec<Event<'parse>>, Self::ErrorType> {
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

#[derive(Clone)]
pub struct Cmark<'parse> {
    events: Vec<Event<'parse>>,
}

impl<'parse> Verbatim<'parse> for Cmark<'parse> {
    type ErrorType = pulldown_cmark_to_cmark::Error;

    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn consume(&mut self, event: Event<'parse>) {
        self.events.push(event);
    }

    fn render(&self) -> Result<String, Self::ErrorType> {
        let mut buf = String::new();

        pulldown_cmark_to_cmark::cmark_with_options(
            self.events.clone().into_iter(),
            &mut buf,
            pulldown_cmark_to_cmark::Options {
                increment_ordered_list_bullets: true,
                ..pulldown_cmark_to_cmark::Options::default()
            },
        )
        .map(|_| buf)
    }
}
