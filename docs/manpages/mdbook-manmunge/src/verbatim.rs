use std::borrow::Cow;
use std::fmt::{Display, Write};

use itertools::Itertools;
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

// TODO Can the following From impls be abstracted?...
impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error {
            message: value.to_string(),
        }
    }
}

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

impl Cmark<'_> {
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

enum Alignment {
    Left,
    Right,
    Centre,
}

struct Column {
    header: String,
    alignment: Alignment,
    data: Vec<String>,

    // Dimensions are computed on the fly to avoid recomputing
    width: usize,
    height: usize,
}

impl Column {
    fn new<T: Into<String>>(header: T, alignment: Alignment) -> Self {
        let header = header.into();
        let width = header.len();

        Column {
            header,
            alignment,
            data: Vec::new(),
            width,
            height: 0,
        }
    }

    /// Insert new column row, updating width as necessary
    fn insert<T: Into<String>>(&mut self, datum: T) {
        let datum = datum.into();
        let width = self.width.max(datum.len());

        self.data.push(datum);
        self.width = width;
        self.height += 1;
    }

    /// Maximum width (characters) of the column row data (including the header)
    fn width(&self) -> usize {
        self.width
    }

    /// Rows in the column (excluding the header)
    fn height(&self) -> usize {
        self.height
    }
}

enum Row {
    Header,
    Rule,
    Data(usize),
}

pub struct Table {
    columns: Vec<Column>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    fn height(&self) -> usize {
        self.columns.iter().map(Column::height).next().unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.height() == 0
    }

    /// Are all columns the same length?
    fn is_consistent(&self) -> bool {
        let first = self.height();

        self.columns
            .iter()
            .map(Column::height)
            .all(|length| length == first)
    }

    fn write_row(&self, output: &mut impl Write, row: Row) -> std::fmt::Result {
        writeln!(
            output,
            "| {} |",
            self.columns
                .iter()
                .map(|column| {
                    let width = column.width;
                    let content = match row {
                        Row::Header => &column.header,
                        Row::Rule => &"-".repeat(column.width),
                        Row::Data(idx) => &column.data[idx],
                    };

                    match column.alignment {
                        Alignment::Left => {
                            format!("{content:<width$}")
                        }

                        Alignment::Right => {
                            format!("{content:>width$}")
                        }

                        Alignment::Centre => {
                            format!("{content:^width$}")
                        }
                    }
                })
                .join(" | ")
        )
    }
}

impl<'parse> Verbatim<'parse> for Table {
    fn consume(&mut self, event: Event<'parse>) {
        todo!()
    }

    fn render(&self) -> Result<String, Error> {
        if self.is_empty() {
            return Err("Table has no columns or rows".into());
        }

        if !self.is_consistent() {
            return Err("Table has inconsistent column lengths".into());
        }

        let mut output = String::new();
        self.write_row(&mut output, Row::Header)?;
        self.write_row(&mut output, Row::Rule)?;
        for row in 0..self.height() {
            self.write_row(&mut output, Row::Data(row))?;
        }

        Ok(output)
    }
}
