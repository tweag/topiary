use std::borrow::Cow;
use std::fmt::Write;

use itertools::Itertools;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

/* Error handling ********************************************************************************/

#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

trait ErrorString: ToString {}

impl<T: ErrorString> From<T> for Error {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl ErrorString for &str {}
impl ErrorString for std::fmt::Error {}
impl ErrorString for pulldown_cmark_to_cmark::Error {}

/* Escaping **************************************************************************************/

trait Escape {
    fn escape_backslashes(&self) -> Cow<str>;
}

impl Escape for String {
    fn escape_backslashes(&self) -> Cow<str> {
        // FIXME Only escape backslashes
        self.replace("\\", "\\\\").into()
    }
}

/* Verbatim trait ********************************************************************************/

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

/* Verbatim Markdown rendering *******************************************************************/

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

/* Verbatim table rendering **********************************************************************/

/// Table cell alignment
enum Alignment {
    Left,
    Right,
    Centre,
}

/// Table column
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

        Self {
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

/// Table row type
enum Row {
    Header,
    Rule,
    Data(usize),
}

/// Table
struct Table {
    columns: Vec<Column>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    /// The height of the tallest column
    fn height(&self) -> usize {
        self.columns.iter().map(Column::height).max().unwrap_or(0)
    }

    /// Does the table have any columns or rows?
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

    /// Write table row to output
    fn write_row(&self, output: &mut impl Write, row: Row) -> std::fmt::Result {
        // Empty cell contents, when columns have ragged lengths
        let empty = "".to_string();

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
                        Row::Data(idx) => column.data.get(idx).unwrap_or(&empty),
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

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            log::warn!("Formatting table with no columns or rows");
        }

        if !self.is_consistent() {
            log::warn!("Formatting table with ragged column lengths");
        }

        self.write_row(f, Row::Header)?;
        self.write_row(f, Row::Rule)?;
        for row in 0..self.height() {
            self.write_row(f, Row::Data(row))?;
        }

        Ok(())
    }
}

/// Consume pulldown_cmark table events and render them as a formatted table; similar to a Markdown
/// table, with equal column spacing and padding
pub struct CmarkTable<'parse> {
    table: Table,
    current_cell: Vec<Event<'parse>>,
}

impl CmarkTable<'_> {
    pub fn new() -> Self {
        Self {
            table: Table::new(),
            current_cell: Vec::new(),
        }
    }
}

impl<'parse> Verbatim<'parse> for CmarkTable<'parse> {
    fn consume(&mut self, event: Event<'parse>) {
        todo!()
    }

    fn render(&self) -> Result<String, Error> {
        Ok(self.table.to_string())
    }
}
