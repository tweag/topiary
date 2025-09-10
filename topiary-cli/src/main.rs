mod cli;
mod error;
mod fs;
mod io;
mod language;
mod visualisation;

use std::{
    error::Error,
    io::{BufReader, BufWriter, Write},
    process::ExitCode,
};

use error::Benign;
use tabled::{settings::Style, Table};
use topiary_config::source::Source;
use topiary_core::{check_query_coverage, formatter, Operation};

use crate::{
    cli::Commands,
    error::{CLIError, CLIResult, TopiaryError},
    io::{read_input, Inputs, OutputFile},
    language::LanguageDefinitionCache,
};

use miette::{NamedSource, Report};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        if !e.benign() {
            print_error(&e)
        }
        return e.into();
    }

    ExitCode::SUCCESS
}

async fn run() -> CLIResult<()> {
    let args = cli::get_args()?;

    let file_config = &args.global.configuration;
    let (config, nickel_config) =
        topiary_config::Configuration::fetch(args.global.merge_configuration, file_config)?;

    // Delegate by subcommand
    match args.command {
        Commands::Format {
            tolerate_parsing_errors,
            skip_idempotence,
            inputs,
        } => {
            let inputs = Inputs::new(&config, &inputs);
            let cache = LanguageDefinitionCache::new();

            let (_, mut results) = async_scoped::TokioScope::scope_and_block(|scope| {
                for input in inputs {
                    scope.spawn(async {
                        let result: CLIResult<()> = match input {
                            Ok(input) => {
                                let language = cache.fetch(&input).await?;
                                let output = OutputFile::try_from(&input)?;

                                log::info!(
                                    "Formatting {}, as {} using {}, to {}",
                                    input.source(),
                                    input.language().name,
                                    input.query(),
                                    output
                                );

                                let mut buf_output = BufWriter::new(output);

                                {
                                    // NOTE This newly opened scope is important! `buf_input` takes
                                    // ownership of `input`, which -- upon reading -- contains an
                                    // open file handle. We need to close this file, by dropping
                                    // `buf_input`, before we attempt to persist our output.
                                    // Otherwise, we get an exclusive lock problem on Windows.
                                    let mut buf_input = BufReader::new(input);

                                    formatter(
                                        &mut buf_input,
                                        &mut buf_output,
                                        &language,
                                        Operation::Format {
                                            skip_idempotence,
                                            tolerate_parsing_errors,
                                        },
                                    )
                                    .map_err(|e| {
                                        e.with_location(format!("{}", buf_input.get_ref().source()))
                                    })?;
                                }

                                buf_output.into_inner()?.persist()?;

                                Ok(())
                            }

                            // This happens when the input resolver cannot establish an input
                            // source, language or query file.
                            Err(error) => Err(error),
                        };

                        if let Err(error) = &result {
                            // By this point, we've lost any reference to the original
                            // input; we trust that it is embedded into `error`.
                            log::warn!("Skipping: {error}");
                        }

                        result
                    });
                }
            });

            if results.len() == 1 {
                // If we just had one input, then handle errors as normal
                results.remove(0)??
            } else if results
                .iter()
                .any(|result| matches!(result, Err(_) | Ok(Err(_))))
            {
                // For multiple inputs, bail out if any failed with a "multiple errors" failure
                return Err(TopiaryError::Bin(
                    "Processing of some inputs failed; see warning logs for details".into(),
                    Some(CLIError::Multiple),
                ));
            }
        }

        Commands::Visualise { format, input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            let input = Inputs::new(&config, &input).next().unwrap()?;
            let output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let language = input.to_language().await?;

            log::info!(
                "Visualising {}, as {}, to {}",
                input.source(),
                input.language().name,
                output
            );

            let mut buf_input = BufReader::new(input);
            let mut buf_output = BufWriter::new(output);

            formatter(
                &mut buf_input,
                &mut buf_output,
                &language,
                Operation::Visualise {
                    output_format: format.into(),
                },
            )
            .map_err(|e| e.with_location(format!("{}", buf_input.get_ref().source())))?;
        }

        Commands::Config { show_sources: true } => {
            let sources = Source::config_sources(file_config)
                .map(|(hint, source)| {
                    let (source, exists) = source
                        .map(|s| {
                            let exists = match s {
                                Source::File(ref file) => file.exists(),
                                Source::Builtin => true,
                            };

                            (format!("{s}"), exists)
                        })
                        .unwrap_or_else(|| ("<unknown>".to_string(), false));
                    (hint, source, exists)
                })
                .collect::<Vec<_>>();

            let mut table = Table::builder(sources);
            table.remove_record(0);
            table.insert_record(0, ["source", "path", "exists"]);
            println!("{}", table.build().with(Style::modern_rounded()));
        }

        Commands::Config {
            show_sources: false,
        } => {
            // Output the collated nickel configuration.
            // Don't fail on error but merely log the event since the original `nickel_config` is
            // already valid.
            #[cfg(feature = "nickel")]
            if io::format_config(&config, &nickel_config)
                .await
                .inspect_err(|e| {
                    // nickel may not be present in user config so log as info
                    log::info!("Config formatting error: {}", e);
                })
                .is_ok()
            {
                return Ok(());
            }
            println!("{nickel_config}");
        }

        Commands::Prefetch { force, language } => match language {
            Some(l) => config.prefetch_language(l, force)?,
            _ => config.prefetch_languages(force)?,
        },

        Commands::Coverage { input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            let input = Inputs::new(&config, &input).next().unwrap()?;
            let output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let language = input.to_language().await?;

            log::info!(
                "Checking query coverage of {}, as {}",
                input.source(),
                input.language().name,
            );

            let mut buf_input = BufReader::new(input);
            let mut buf_output = BufWriter::new(output);

            let input_content = read_input(&mut buf_input)?;

            let coverage_data =
                check_query_coverage(&input_content, &language.query, &language.grammar)
                    .map_err(|e| e.with_location(buf_input.get_ref().source().to_string()))?;
            let coverage_res = coverage_data.get_result();

            let query_source = NamedSource::new(
                buf_input.get_ref().query.to_string(),
                language.query.query_content,
            )
            .with_language(&language.name);
            write!(
                &mut buf_output,
                "{:?}",
                Report::new(coverage_data).with_source_code(query_source)
            )?;

            coverage_res?;
        }

        Commands::Completion { shell } => {
            // The CLI parser fails if no shell is provided/detected, so it's safe to unwrap here
            cli::completion(shell.unwrap());
        }
    }

    Ok(())
}

fn print_error(e: &dyn Error) {
    log::error!("{e}");
    if let Some(source) = e.source() {
        log::error!("Cause: {source}");
    }
}
