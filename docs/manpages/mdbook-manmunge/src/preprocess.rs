use std::io;

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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
                Err(error) => eprintln!("Could not process chapter: {error}"),
            }
        });

        Ok(book)
    }
}

fn rewrite_chapter(chapter: &mut Chapter) -> Result<String, Error> {
    let mut buf = String::with_capacity(chapter.content.len());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&chapter.content, options);

    let mut strip_link = false;
    let events = parser.filter(|event| match event {
        Event::Start(Tag::Link { dest_url, .. }) => {
            let url = Url::parse(dest_url);

            match url {
                Ok(_) => true,

                Err(ParseError::RelativeUrlWithoutBase) => {
                    eprintln!("{}: Stripping relative URL {dest_url}", chapter.name);
                    strip_link = true;
                    false
                }

                Err(error) => {
                    eprintln!(
                        "{}: Stripping unparsable URL {dest_url} ({error}).",
                        chapter.name
                    );
                    strip_link = true;
                    false
                }
            }
        }

        Event::End(TagEnd::Link) => {
            let keep_link = !strip_link;
            strip_link = false;

            keep_link
        }

        _ => true,
    });

    Ok(pulldown_cmark_to_cmark::cmark(events, &mut buf).map(|_| buf)?)
}

pub fn preprocess() -> Result<(), Error> {
    let preprocessor = MdbookMunge::new();
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
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
