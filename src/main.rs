static TEST_FILE: &str = "tests/json.json";
static QUERY_FILE: &str = "languages/queries/json.scm";

use std::error::Error;
use std::io::stdout;
use std::path::Path;
use tree_sitter_formatter::formatter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut out = stdout();
    formatter(Path::new(TEST_FILE), Path::new(QUERY_FILE), &mut out)?;
    Ok(())
}
