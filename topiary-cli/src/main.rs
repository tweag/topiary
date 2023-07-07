mod configuration;
mod error;
mod output;
mod visualise;

use std::{
    eprintln,
    error::Error,
    fs::File,
    io::{stdin, BufReader, BufWriter, Read},
    path::PathBuf,
    process::ExitCode,
};

use clap::{ArgGroup, Parser};
use configuration::parse_configuration;

use crate::{
    error::{CLIError, CLIResult, TopiaryError},
    output::OutputFile,
    visualise::Visualisation,
};
use topiary::{formatter, Language, Operation, SupportedLanguage, TopiaryQuery};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// Require at least one of --language or --input-files (n.b., language > input)
#[command(group(ArgGroup::new("rule").multiple(true).required(true).args(&["language", "input_files"]),))]
struct Args {
    /// Which language to parse and format
    #[arg(short, long, value_enum, display_order = 1)]
    language: Option<SupportedLanguage>,

    /// Path to an input file or multiple input files. If omitted, or equal
    /// to "-", read from standard input. If multiple files are provided,
    /// `in_place` is assumed.
    #[arg(short = 'f', long, num_args = 0.., display_order = 2, default_values_t = ["-".to_string()])]
    input_files: Vec<String>,

    /// Which query file to use
    #[arg(short, long, display_order = 3)]
    query: Option<PathBuf>,

    /// Path to an output file. If omitted, or equal to "-", write to standard
    /// output.
    #[arg(short, long, display_order = 4)]
    output_file: Option<String>,

    /// Format the input files in place.
    #[arg(short, long, requires = "input_files", display_order = 5)]
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

    /// Output the full configuration to stderr before continuing
    #[arg(long, display_order = 8)]
    output_configuration: bool,

    /// Format as much as possible even if some of the input causes parsing errors
    #[arg(short, long, display_order = 9)]
    tolerate_parsing_errors: bool,

    /// Override all configuration with the provided file
    #[arg(long, env = "TOPIARY_CONFIGURATION_OVERRIDE", display_order = 10)]
    configuration_override: Option<PathBuf>,
}

// /// Collects all the values needed for the eventual formatting. This helper
// /// struct just makes it easy to collect them all in a Vec.
// /// If `--in-place` was specified or if `--input-files` was
// /// given more than one file, the input and output will be the same file and
// /// the entire FormatStruct will be placed in a Vector. In all other cases the vector
// /// will still be created, but it will be a singleton vector.
// struct FormatStruct<'a> {
//     input: &'a dyn Read,
//     output: BufWriter<OutputFile>,
//     language: &'a Language,
// }

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        print_error(&e);
        return e.into();
    }

    ExitCode::SUCCESS
}

async fn run() -> CLIResult<()> {
    env_logger::init();

    // Restructure Args to match our expected behaviour
    let args = {
        let mut args = Args::parse();

        // Remove duplicates from the input_files, (among other things, avoids being able to pass "-" twice)
        args.input_files.sort_unstable();
        args.input_files.dedup();

        args
    };

    let configuration = parse_configuration(args.configuration_override)?;

    if args.output_configuration {
        eprintln!("{:#?}", configuration);
    }

    let io_files: Vec<(String, String)> = if args.in_place || args.input_files.len() > 1 {
        args.input_files
            .iter()
            .map(|f| (f.clone(), f.clone()))
            .collect()
    } else {
        // Clap guarantees our input_files is non-empty
        vec![(
            args.input_files.first().unwrap().clone(),
            match args.output_file.as_deref() {
                Some("-") | None => String::from("-"),
                Some(f) => String::from(f),
            },
        )]
    };

    type IoFile = (
        String,
        String,
        Language,
        Option<PathBuf>,
        CLIResult<PathBuf>,
    );

    // Add the language and query Path to the io_files
    let io_files: Vec<IoFile> = io_files
        .into_iter()
        // Add the appropriate language to all of the tuples
        .map(|(i, o)| {
            let language = if let Some(language) = args.language {
                language.to_language(&configuration).clone()
            } else {
                Language::detect(&i, &configuration)?.clone()
            };

            let query_path = if let Some(query) = &args.query {
                Ok(query.clone())
            } else {
                language.query_file()
            }
            .map_err(TopiaryError::Lib);

            Ok((i, o, language, args.query.clone(), query_path))
        })
        .collect::<CLIResult<Vec<_>>>()?;

    // Converts the simple types into arguments we can pass to the `formatter` function
    // _ holds the tree_sitter_facade::Language
    let fmt_args: Vec<(String, String, Language, _, TopiaryQuery)> =
        futures::future::try_join_all(io_files.into_iter().map(
            |(i, o, language, query_arg, query_path)| async move {
                let grammar = language.grammar().await?;

                let query = query_path
                    .and_then(|query_path| {
                        {
                            let mut reader = BufReader::new(File::open(query_path)?);
                            let mut contents = String::new();
                            reader.read_to_string(&mut contents)?;
                            Ok(contents)
                        }
                        .map_err(|e| {
                            TopiaryError::Bin(
                                "Could not open query file".into(),
                                Some(CLIError::IOError(e)),
                            )
                        })
                    })
                    .and_then(|query_content: String| {
                        Ok(TopiaryQuery::new(&grammar, &query_content)?)
                    })
                    .or_else(|e| {
                        // If we weren't able to read the query file, and the user didn't
                        // request a specific query file, we should fall back to the built-in
                        // queries.
                        if query_arg.is_none() {
                            log::info!(
                                "No language file found for {language:?}. Will use built-in query."
                            );
                            Ok((&language).try_into()?)
                        } else {
                            Err(e)
                        }
                    })?;

                Ok::<_, TopiaryError>((i, o, language, grammar, query))
            },
        ))
        .await?;

    // The operation needs not be part of the Vector of Structs because it is the same for every formatting instance
    let operation = if let Some(visualisation) = args.visualise {
        Operation::Visualise {
            output_format: visualisation.into(),
        }
    } else {
        Operation::Format {
            skip_idempotence: args.skip_idempotence,
            tolerate_parsing_errors: args.tolerate_parsing_errors,
        }
    };

    let tasks: Vec<_> = fmt_args
        .into_iter()
        .map(|(input, output, language, grammar, query)| -> tokio::task::JoinHandle<Result<(), TopiaryError>> {
            tokio::spawn(async move {
                    let mut input: Box<dyn Read> = match input.as_str() {
                        "-" => Box::new(stdin()),
                        file => Box::new(BufReader::new(File::open(file)?)),
                    };
                    let mut output: BufWriter<OutputFile> = BufWriter::new(OutputFile::new(&output)?);

                    formatter(
                        &mut input,
                        &mut output,
                        &query,
                        &language,
                        &grammar,
                        operation,
                    )?;

                    output.into_inner()?.persist()?;

                    Ok(())
            })
        })
        .collect();

    for task in tasks {
        // The await results in a `Result<Result<(), TopiaryError>, JoinError>`.
        // The first ? concerns the `JoinError`, the second one the `TopiaryError`.
        task.await??;
    }

    Ok(())
}

fn print_error(e: &dyn Error) {
    log::error!("{e}");
    if let Some(source) = e.source() {
        log::error!("Cause: {source}");
    }
}
