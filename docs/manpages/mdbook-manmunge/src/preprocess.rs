use std::io;

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use semver::{Version, VersionReq};

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

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        // We don't have any arguments yet, but if/when we do, they can be extracted from
        // `ctx.config.get_preprocessor(self.name())`
        Ok(book)
    }
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
