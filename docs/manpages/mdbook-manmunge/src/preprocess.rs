use std::io;

use mdbook::BookItem;
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use semver::{Version, VersionReq};
use url::{ParseError, Url};

use crate::error::Error;
use crate::verbatim::{Cmark, CmarkTable, Verbatim};

struct MdbookMunge;

impl MdbookMunge {
    fn new() -> Self {
        Self
    }
}

impl Preprocessor for MdbookMunge {
    fn name(&self) -> &str {
        "manmunge"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> mdbook::errors::Result<Book> {
        // We don't have any arguments yet, but if/when we do, they can be extracted from
        // `ctx.config.get_preprocessor(self.name())`

        book.for_each_mut(|item| {
            let BookItem::Chapter(chapter) = item else {
                return;
            };
            if chapter.is_draft_chapter() {
                return;
            }

            match rewrite_chapter(chapter) {
                Ok(result) => chapter.content = result,
                Err(error) => log::error!("{}: Could not process chapter ({error})", chapter.name),
            }
        });

        Ok(book)
    }
}

fn event_type<'parse>(event: &Event<'parse>) -> &'static str {
    match event {
        Event::Start(Tag::List(Some(_))) | Event::End(TagEnd::List(true)) => "ordered list",
        Event::Start(Tag::List(None)) | Event::End(TagEnd::List(false)) => "unordered list",
        Event::Start(Tag::Table(_)) | Event::End(TagEnd::Table) => "table",
        _ => unreachable!(),
    }
}

fn rewrite_chapter(chapter: &mut Chapter) -> Result<String, Error> {
    let mut buf = String::with_capacity(chapter.content.len());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&chapter.content, options);

    let mut strip_h1 = false;
    let mut strip_link = false;
    let mut verbatim: Option<Box<dyn Verbatim>> = None;

    let events = parser
        .flat_map(|event| match event {
            // String links with relative and unparsable URLs
            Event::Start(Tag::Link { ref dest_url, .. }) => {
                let url = Url::parse(dest_url);

                match url {
                    Ok(_) => vec![Some(event)],

                    Err(ParseError::RelativeUrlWithoutBase) => {
                        log::info!("{}: Stripping relative URL {dest_url}", chapter.name);
                        strip_link = true;
                        vec![None]
                    }

                    Err(error) => {
                        log::warn!(
                            "{}: Stripping unparsable URL {dest_url} ({error}).",
                            chapter.name
                        );
                        strip_link = true;
                        vec![None]
                    }
                }
            }

            Event::End(TagEnd::Link) => {
                if strip_link {
                    strip_link = false;
                    vec![None]
                } else {
                    vec![Some(event)]
                }
            }

            // Strip top-level headings, as mdbook-man uses the chapter heading from SUMMARY.md
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                log::info!("{}: Stripping H1", chapter.name);
                strip_h1 = true;
                vec![None]
            }

            Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                strip_h1 = false;
                vec![None]
            }

            _ if strip_h1 => vec![None],

            // Slurp up lists and tables and then rewrite them as Markdown within a code fence
            Event::Start(Tag::List(_)) | Event::Start(Tag::Table(_)) => {
                if verbatim.is_some() {
                    log::error!(
                        "{}: Nested verbatim structure found; skipping.",
                        chapter.name
                    );
                } else {
                    log::info!("{}: Slurping in {}", chapter.name, event_type(&event));

                    let mut verbatim_events: Box<dyn Verbatim> = match event {
                        Event::Start(Tag::List(_)) => Box::new(Cmark::new()),
                        Event::Start(Tag::Table(_)) => Box::new(CmarkTable::new()),
                        _ => unreachable!(),
                    };

                    match verbatim_events.consume(event) {
                        Ok(()) => verbatim = Some(verbatim_events),

                        Err(error) => {
                            log::error!("{}: Could not consume Markdown; {error}", chapter.name)
                        }
                    }
                }

                vec![None]
            }

            Event::End(TagEnd::List(_)) | Event::End(TagEnd::Table) => {
                if let Some(mut verbatim_events) = verbatim.take() {
                    log::info!(
                        "{}: Regurgitating {} structure verbatim",
                        chapter.name,
                        event_type(&event)
                    );

                    match verbatim_events.consume(event) {
                        Ok(()) => match verbatim_events.emit() {
                            Ok(events) => events.iter().map(|event| Some(event.clone())).collect(),

                            Err(error) => {
                                log::error!("{}: Could not regurgitate; {error}", chapter.name);
                                vec![None]
                            }
                        },

                        Err(error) => {
                            log::error!("{}: Could not consume Markdown; {error}", chapter.name);
                            vec![None]
                        }
                    }
                } else {
                    log::error!(
                        "{}: Unexpected end of verbatim structure found.",
                        chapter.name
                    );

                    vec![None]
                }
            }

            _ if verbatim.is_some() => {
                if let Some(verbatim_events) = verbatim.as_mut() {
                    if let Err(error) = verbatim_events.consume(event) {
                        log::error!("{}: Could not consume Markdown; {error}", chapter.name);
                    }
                }
                vec![None]
            }

            // Convert soft breaks into an explicit space, to prevent hard wrapped lines in the
            // input getting smushed together in the output
            Event::SoftBreak => {
                vec![Some(Event::Text("\\ ".into()))]
            }

            // Everything else
            _ => vec![Some(event)],
        })
        // We have to flatten again so Some(e) -> e, and Nones disappear
        .flatten();

    Ok(pulldown_cmark_to_cmark::cmark(events, &mut buf).map(|_| buf)?)
}

pub fn preprocess() -> Result<(), Error> {
    let preprocessor = MdbookMunge::new();
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        log::warn!(
            "Warning: {} uses mdBook v{}, but is being called from v{}",
            preprocessor.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;

    Ok(())
}
