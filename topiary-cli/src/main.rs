mod cli;
mod configuration;
mod error;
mod output;
mod visualisation;

use std::{
    eprintln,
    error::Error,
    fs::File,
    io::{stdin, BufReader, BufWriter, Read},
    path::PathBuf,
    process::ExitCode,
};

use crate::{
    cli::Commands,
    error::{CLIError, CLIResult, TopiaryError},
    output::OutputFile,
};
use topiary::{formatter, Language, Operation, TopiaryQuery};

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

    let args = cli::get_args()?;
    let config = configuration::parse(
        args.global.configuration,
        // The collation value is always set, so we can safely unwrap
        args.global.configuration_collation.unwrap(),
    )?;

    // Delegate by subcommand
    match args.command {
        Commands::Fmt {
            parse,
            skip_idempotence,
            language,
            query,
            files,
        } => {
            todo!();
        }

        Commands::Vis {
            parse,
            format,
            language,
            query,
            file,
        } => {
            todo!();
        }

        Commands::Cfg => {
            println!("{}", config);
        }
    }

    Ok(())
}

/*
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
*/

fn print_error(e: &dyn Error) {
    log::error!("{e}");
    if let Some(source) = e.source() {
        log::error!("Cause: {source}");
    }
}
