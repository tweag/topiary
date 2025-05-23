use std::borrow::Cow;
use std::fmt::Write;

use crate::error::Error;

use itertools::Itertools;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

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
    fn consume(&mut self, event: Event<'parse>) -> Result<(), Error>;

    /// Drain the consumed events and render them as verbatim plain text
    fn drain_to_string(&mut self) -> Result<String, Error>;

    /// Emit pulldown_cmark events within a code fence
    fn emit(&mut self) -> Result<Vec<Event<'parse>>, Error> {
        Ok(vec![
            // Opening code fence
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("".into()))),
            // Verbatim contents
            Event::Text(
                self.drain_to_string()?
                    .escape_backslashes()
                    .into_owned()
                    .into(),
            ),
            // Closing code fence
            Event::End(TagEnd::CodeBlock),
        ])
    }
}

/* Verbatim Markdown rendering *******************************************************************/

/// Consume pulldown_cmark events and render them as Markdown
pub struct Cmark<'parse> {
    events: Vec<Event<'parse>>,
}

impl Cmark<'_> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl<'parse> Verbatim<'parse> for Cmark<'parse> {
    fn consume(&mut self, event: Event<'parse>) -> Result<(), Error> {
        self.events.push(event);
        Ok(())
    }

    fn drain_to_string(&mut self) -> Result<String, Error> {
        let mut buf = String::new();

        pulldown_cmark_to_cmark::cmark_with_options(
            self.events.drain(..),
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

impl From<pulldown_cmark::Alignment> for Alignment {
    fn from(value: pulldown_cmark::Alignment) -> Self {
        match value {
            pulldown_cmark::Alignment::Right => Alignment::Right,
            pulldown_cmark::Alignment::Center => Alignment::Centre,
            _ => Alignment::Left,
        }
    }
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

/// Columnar table
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

    /// Add (empty) column to table
    fn add_column<T: Into<String>>(&mut self, header: T, alignment: Alignment) {
        let column = Column::new(header, alignment);
        self.columns.push(column);
    }

    /// Insert datum into column by its index
    fn try_insert<T: Into<String>>(&mut self, column_idx: usize, datum: T) -> Result<(), Error> {
        if let Some(column) = self.columns.get_mut(column_idx) {
            column.insert(datum);
            Ok(())
        } else {
            Err(format!("Cannot insert datum into column {column_idx}: Out of bounds").into())
        }
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
/// table, per Cmark (above), but with equal column spacing and correct cell alignment (see
/// Byron/pulldown-cmark-to-cmark#105).
///
/// For example, Cmark will output something like this:
///     ```
///     |Heading|Another Heading|xyz|
///     |:------|--------------:|:-:|
///     |foo|bar|a|
///     |quux|abc123|Centred|
///     ```
/// Whereas CmarkTable will output something like this:
///     ```
///     | Heading | Another Heading |   xyz   |
///     | ------- | --------------- | ------- |
///     | foo     |             bar |    x    |
///     | quux    |          abc123 | Centred |
///     ```
pub struct CmarkTable<'parse> {
    table: Table,

    // State to build the table, cell-by-cell
    align_buffer: Vec<Alignment>,
    in_header: bool,
    col_cursor: usize,
    cell_buffer: Cmark<'parse>,
}

impl CmarkTable<'_> {
    pub fn new() -> Self {
        Self {
            table: Table::new(),

            align_buffer: Vec::new(),
            in_header: false,
            col_cursor: 0,
            cell_buffer: Cmark::new(),
        }
    }

    /// Render the contents of the current cell buffer, emptying it in the process
    fn render_cell(&mut self) -> Result<String, Error> {
        self.cell_buffer.drain_to_string()
    }
}

impl<'parse> Verbatim<'parse> for CmarkTable<'parse> {
    // Cmark table events are emitted like so:
    //
    //   Start(Table([Alignments..]))
    //     Start(TableHead)
    //       Start(TableCell)
    //         <Cell content events>
    //       End(TableCell)
    //       <x Alignments...>
    //     End(TableHead)
    //     Start(TableRow)
    //       Start(TableCell)
    //         <Cell content events>
    //       End(TableCell)
    //       <x Alignments...>
    //     End(TableRow)
    //     <x Rows...>
    //   End(Table)
    //
    // These need slurping into our CmarkTable struct
    fn consume(&mut self, event: Event<'parse>) -> Result<(), Error> {
        match event {
            Event::Start(Tag::Table(alignments)) => {
                self.align_buffer = alignments.into_iter().map(Alignment::from).collect();
            }

            Event::Start(Tag::TableHead) => {
                self.in_header = true;
                self.col_cursor = 0;
            }

            Event::End(TagEnd::TableHead) => {
                self.in_header = false;
            }

            Event::Start(Tag::TableRow) => {
                self.col_cursor = 0;
            }

            Event::End(TagEnd::TableCell) => {
                let cell_contents = self.cell_buffer.drain_to_string()?;

                // TODO Push rendered contents into appropriate column, or create a new column
                // if it doesn't yet exist
            }

            // Consume everything else into the current cell buffer
            event => {
                if !is_table_event(&event) {
                    self.cell_buffer.consume(event)?;
                }
            }
        }

        Ok(())
    }

    fn drain_to_string(&mut self) -> Result<String, Error> {
        Ok(self.table.to_string())
    }
}

/// Is the Cmark event a table event?
fn is_table_event(event: &Event) -> bool {
    matches!(
        event,
        Event::Start(Tag::Table(_))
            | Event::Start(Tag::TableHead)
            | Event::Start(Tag::TableRow)
            | Event::Start(Tag::TableCell)
            | Event::End(TagEnd::Table)
            | Event::End(TagEnd::TableHead)
            | Event::End(TagEnd::TableRow)
            | Event::End(TagEnd::TableCell)
    )
}
