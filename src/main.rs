use clap::{ArgEnum, ArgGroup, Parser};
use std::{
    error::Error,
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter},
    path::PathBuf,
};
use topiary::{formatter, FormatterResult, Language};

#[derive(ArgEnum, Clone, Copy, Debug)]
enum SupportedLanguage {
    Json,
    Toml,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

impl From<SupportedLanguage> for Language {
    fn from(language: SupportedLanguage) -> Self {
        match language {
            SupportedLanguage::Json => Language::Json,
            SupportedLanguage::Toml => Language::Toml,
            SupportedLanguage::Ocaml => Language::Ocaml,
            SupportedLanguage::OcamlImplementation => Language::OcamlImplementation,
            SupportedLanguage::OcamlInterface => Language::OcamlInterface,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
// Require at least one of --language, --query or --input-file (n.b., query > language > input)
#[clap(group(ArgGroup::new("rule").multiple(true).required(true).args(&["language", "query", "input-file"]),))]
struct Args {
    /// Which language to parse and format
    #[clap(short, long, arg_enum)]
    language: Option<SupportedLanguage>,

    /// Which query file to use
    #[clap(short, long)]
    query: Option<PathBuf>,

    /// Do not check that formatting twice gives the same output
    #[clap(short, long)]
    skip_idempotence: bool,

    /// Path to an input file. If omitted, or equal to "-", read from standard
    /// input.
    #[clap(short, long)]
    input_file: Option<String>,

    /// Path to an output file. If omitted, or equal to "-", write to standard
    /// output.
    #[clap(short, long)]
    output_file: Option<String>,
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

    // The as_deref() gives us an Option<&str>, which we can match against
    // string literals
    let mut input: Box<dyn std::io::Read> = match args.input_file.as_deref() {
        Some("-") | None => Box::new(stdin()),
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
    };

    let mut output: Box<dyn std::io::Write> = match args.output_file.as_deref() {
        Some("-") | None => Box::new(stdout()),
        Some(file) => Box::new(BufWriter::new(File::open(file)?)),
    };

    let language: Option<Language> = if let Some(language) = args.language {
        Some(language.into())
    } else if let Some(filename) = args.input_file.as_deref() {
        Some(Language::detect(filename)?)
    } else {
        // At this point, Clap ensures that args.query must be present.
        // We will read the language from the query file later.
        None
    };

    let query_path = if let Some(query) = args.query {
        query
    } else if let Some(language) = language {
        // Deduce the query file from the language, if the argument is missing
        Language::query_path(language)
    } else {
        // Clap ensures we won't get here
        unreachable!();
    };

    let mut query = BufReader::new(File::open(query_path)?);

    formatter(
        &mut input,
        &mut output,
        &mut query,
        language,
        args.skip_idempotence,
    )?;

    Ok(())
}

fn print_error(e: &dyn Error) {
    eprintln!("Error: {}", e);
    if let Some(source) = e.source() {
        eprintln!("  Caused by: {}", source);
    }
}
