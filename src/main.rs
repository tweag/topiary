use clap::{ArgEnum, ArgGroup, Parser};
use std::{
    error::Error,
    fs::File,
    io::{stdin, stdout, BufReader},
};
use tree_sitter_formatter::{formatter, FormatterResult};

#[derive(ArgEnum, Clone, Copy, Debug)]
enum SupportedLanguage {
    Json,
    Toml,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
// Require either --language or --query, but not both.
#[clap(group(ArgGroup::new("rule").required(true).args(&["language", "query"]),))]
struct Args {
    /// Which language to parse and format
    #[clap(short, long, arg_enum)]
    language: Option<SupportedLanguage>,

    /// Which query file to use
    #[clap(short, long)]
    query: Option<String>,

    /// Do not check that formatting twice gives the same output
    #[clap(short, long)]
    skip_idempotence: bool,
}

fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

fn run() -> FormatterResult<()> {
    env_logger::init();
    let args = Args::parse();
    let mut input = stdin();
    let mut output = stdout();

    let query_path = if let Some(query) = args.query {
        query
    } else if let Some(language) = args.language {
        str::to_lowercase(format!("languages/{language:?}.scm").as_str())
    } else {
        // Clap ensures we won't get here
        unreachable!();
    };

    let mut query = BufReader::new(File::open(query_path)?);

    formatter(&mut input, &mut output, &mut query, args.skip_idempotence)?;

    Ok(())
}

fn print_error(e: &dyn Error) {
    eprintln!("Error: {}", e);
    if let Some(source) = e.source() {
        eprintln!("  Caused by: {}", source);
    }
}
