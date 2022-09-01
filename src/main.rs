use clap::Parser;
use std::error::Error;
use std::io;
use tree_sitter_formatter::{formatter, Language};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which language to parse and format
    #[clap(short, long, arg_enum)]
    language: Language,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();
    let mut input = io::stdin();
    let mut output = io::stdout();

    formatter(&mut input, &mut output, args.language)?;

    Ok(())
}
