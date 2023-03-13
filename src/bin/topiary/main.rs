mod error;
mod output;
mod supported;
mod visualise;

use std::{
    error::Error,
    fs::File,
    io::{stdin, BufReader, BufWriter, Read},
    path::PathBuf,
};

use clap::{ArgGroup, Parser};

use crate::{
    error::CLIResult, output::OutputFile, supported::SupportedLanguage, visualise::Visualisation,
};
use topiary::{formatter, Configuration, FormatterError, Language, Operation};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// Require at least one of --language, --input-file or --query (n.b., language > input > query)
#[command(group(ArgGroup::new("rule").multiple(true).required(true).args(&["language", "input_file", "query"]),))]
struct Args {
    /// Which language to parse and format
    #[arg(short, long, value_enum, display_order = 1)]
    language: Option<SupportedLanguage>,

    /// Path to an input file. If omitted, or equal to "-", read from standard
    /// input.
    #[arg(short = 'f', long, display_order = 2)]
    input_file: Option<String>,

    /// Which query file to use
    #[arg(short, long, display_order = 3)]
    query: Option<PathBuf>,

    /// Path to an output file. If omitted, or equal to "-", write to standard
    /// output.
    #[arg(short, long, display_order = 4)]
    output_file: Option<String>,

    /// Format the input file in place.
    #[arg(short, long, requires = "input_file", display_order = 5)]
    in_place: bool,

    /// Visualise the syntax tree, rather than format.
    #[arg(
        short,
        long,
        value_enum,
        aliases = &["view", "visualize"],
        value_name = "OUTPUT_FORMAT",
        conflicts_with_all = &["in_place", "skip_idempotence"],
        require_equals = true,
        num_args = 0..=1,
        default_missing_value = "json",
        display_order = 6
    )]
    visualise: Option<Visualisation>,

    /// Do not check that formatting twice gives the same output
    #[arg(short, long, display_order = 7)]
    skip_idempotence: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> CLIResult<()> {
    env_logger::init();
    let args = Args::parse();

    // The as_deref() gives us an Option<&str>, which we can match against
    // string literals
    let mut input: Box<(dyn std::io::Read + Send)> = match args.input_file.as_deref() {
        Some("-") | None => Box::new(stdin()),
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
    };

    // NOTE If --in-place is specified, it overrides --output-file
    let mut output = BufWriter::new(if args.in_place {
        // NOTE Clap handles the case when no input file is specified. If the input file is
        // explicitly set to stdin (i.e., -), then --in-place will set the output to stdout; which
        // is not completely weird.
        OutputFile::new(args.input_file.as_deref())?
    } else {
        OutputFile::new(args.output_file.as_deref())?
    });

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
        language.query_file()?
    } else {
        // Clap ensures we won't get here
        unreachable!();
    };

    let mut query_reader = BufReader::new(File::open(query_path)?);
    let mut query = String::new();
    query_reader.read_to_string(&mut query)?;

    let mut configuration = Configuration::parse(&query)?;

    // Replace the language deduced from the query file by the one from the CLI, if any
    if let Some(l) = language {
        configuration.language = l
    }

    let grammars = configuration
        .language
        .grammars()
        .await
        .map_err(|e| FormatterError::Internal("Could not load grammars".into(), Some(e)))?;

    let operation = if let Some(visualisation) = args.visualise {
        Operation::Visualise {
            output_format: visualisation.into(),
        }
    } else {
        Operation::Format {
            skip_idempotence: args.skip_idempotence,
        }
    };

    formatter(
        &mut input,
        &mut output,
        &mut query_reader,
        &grammars,
        &configuration,
        operation,
    )?;

    output.into_inner()?.persist()?;

    Ok(())
}

fn print_error(e: &dyn Error) {
    log::error!("{e}");
    if let Some(source) = e.source() {
        log::error!("Cause: {source}");
    }
}
