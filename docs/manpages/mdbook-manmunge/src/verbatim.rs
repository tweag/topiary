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
    /// Constructor
    fn new() -> Self;

    /// Consume pulldown_cmark event
    fn consume(&mut self, event: Event<'parse>);

    /// Render consumed events as verbatim plain text
    fn render(&self) -> String;

    /// Emit pulldown_cmark events within a code fence
    fn emit(&self) -> Vec<Event<'parse>> {
        vec![
            // Opening code fence
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("".into()))),
            // Verbatim contents
            Event::Text(self.render().escape_backslashes().into_owned().into()),
            // Closing code fence
            Event::End(TagEnd::CodeBlock),
        ]
    }
}

#[derive(Clone)]
pub struct Cmark<'parse> {
    events: Vec<Box<Event<'parse>>>,
}

impl<'parse> Verbatim<'parse> for Cmark<'parse> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn consume(&mut self, event: Event<'parse>) {
        self.events.push(Box::new(event));
    }

    fn render(&self) -> String {
        let mut buf = String::new();

        pulldown_cmark_to_cmark::cmark_with_options(
            self.events.clone().into_iter().map(|boxed| *boxed),
            &mut buf,
            pulldown_cmark_to_cmark::Options {
                increment_ordered_list_bullets: true,
                ..pulldown_cmark_to_cmark::Options::default()
            },
        )
        .map(|_| buf)
        // We assume it's not going to fail because it's effectively a round-trip
        .unwrap()
    }
}
