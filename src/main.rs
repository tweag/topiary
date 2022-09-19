use clap::Parser;
use std::{error::Error, io};
use tree_sitter_formatter::{formatter, Language, Result};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which language to parse and format
    #[clap(short, long, arg_enum)]
    language: Language,

    /// Fail if formatting is not idempotent
    #[clap(short, long)]
    check_idempotence: bool,
}

fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let mut input = io::stdin();
    let mut output = io::stdout();

    formatter(
        &mut input,
        &mut output,
        args.language,
        args.check_idempotence,
    )?;

    Ok(())
}

fn print_error(e: &dyn Error) {
    eprintln!("Error: {}", e);
    if let Some(source) = e.source() {
        eprintln!("  Caused by: {}", source);
    }
}
