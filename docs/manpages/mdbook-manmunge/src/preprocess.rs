use std::io;

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use semver::{Version, VersionReq};
use url::{ParseError, Url};

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

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
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

#[derive(Clone)]
struct VerbatimRewrite<'parse> {
    events: Vec<Box<Event<'parse>>>,
}

impl<'parse> VerbatimRewrite<'parse> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn event_type(event: &Event<'parse>) -> &'parse str {
        match event {
            Event::Start(Tag::List(_)) | Event::End(TagEnd::List(_)) => "list",
            Event::Start(Tag::Table(_)) | Event::End(TagEnd::Table) => "table",
            _ => unreachable!(),
        }
    }

    fn append(&mut self, event: Event<'parse>) {
        self.events.push(Box::new(event));
    }

    // TODO This almost works as expected, however the re-rendering to Markdown is not great in
    // some cases. In particular:
    // * Tables are not padded to have uniform column widths
    // * Back slashes (e.g., in Windows paths) are not escaped
    fn rewrite(self) -> Vec<Option<Event<'parse>>> {
        let mut buf = String::new();

        vec![
            // Open a code fence
            Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
                "".into(),
            )))),
            // Render the consumed events as Markdown
            Some(Event::Text(
                pulldown_cmark_to_cmark::cmark_with_options(
                    self.events.into_iter().map(|boxed| *boxed),
                    &mut buf,
                    pulldown_cmark_to_cmark::Options {
                        increment_ordered_list_bullets: true,
                        ..pulldown_cmark_to_cmark::Options::default()
                    },
                )
                .map(|_| buf)
                // We assume it's not going to fail because it's effectively a round-trip
                .unwrap()
                .into(),
            )),
            // Closing code fence
            Some(Event::End(TagEnd::CodeBlock)),
        ]
    }
}

fn rewrite_chapter(chapter: &mut Chapter) -> Result<String, Error> {
    let mut buf = String::with_capacity(chapter.content.len());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&chapter.content, options);

    let mut strip_h1 = false;
    let mut strip_link = false;
    let mut verbatim: Option<VerbatimRewrite> = None;

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
                    log::info!(
                        "{}: Slurping in {}",
                        chapter.name,
                        VerbatimRewrite::event_type(&event)
                    );

                    let mut v = VerbatimRewrite::new();
                    v.append(event);
                    verbatim = Some(v);
                }

                vec![None]
            }

            Event::End(TagEnd::List(_)) | Event::End(TagEnd::Table) => {
                if verbatim.is_none() {
                    log::error!(
                        "{}: Unexpected end of verbatim structure found.",
                        chapter.name
                    );

                    return vec![None];
                }

                log::info!(
                    "{}: Regurgitating {} structure verbatim",
                    chapter.name,
                    VerbatimRewrite::event_type(&event)
                );

                let mut regurgitate = verbatim.clone().unwrap();
                verbatim = None;

                regurgitate.append(event);
                regurgitate.rewrite()
            }

            _ if verbatim.is_some() => {
                if let Some(verbatim_events) = verbatim.as_mut() {
                    verbatim_events.append(event);
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
