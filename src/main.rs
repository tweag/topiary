use clap::Parser;
use env_logger::{Builder, Env};
use log::LevelFilter;
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
    let args = Args::parse();
    let mut input = io::stdin();
    let mut output = io::stdout();

    let env = Env::new().filter("FORMATTER_LOG");
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Off);
    builder.parse_env(env);
    builder.init();

    formatter(&mut input, &mut output, args.language)?;

    Ok(())
}
