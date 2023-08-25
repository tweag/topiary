mod cli;
mod configuration;
mod error;
mod io;
mod language;
mod visualisation;

use std::{
    error::Error,
    io::{BufReader, BufWriter},
    process::ExitCode,
};

use log::LevelFilter;

use crate::{
    cli::Commands,
    error::{CLIError, CLIResult, TopiaryError},
    io::{Inputs, OutputFile},
    language::LanguageDefinitionCache,
};
use topiary::{formatter, Operation};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        print_error(&e);
        return e.into();
    }

    ExitCode::SUCCESS
}

async fn run() -> CLIResult<()> {
    let args = cli::get_args()?;

    env_logger::Builder::new()
        .filter_level(match args.global.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    let (annotations, config) = configuration::fetch(
        &args.global.configuration,
        // The collation value is always set, so we can safely unwrap
        args.global.configuration_collation.as_ref().unwrap(),
    )?;

    // Delegate by subcommand
    match args.command {
        Commands::Fmt {
            tolerate_parsing_errors,
            skip_idempotence,
            inputs,
        } => {
            let inputs = Inputs::new(&config, &inputs);
            let cache = LanguageDefinitionCache::new();

            let (_, mut results) = async_scoped::TokioScope::scope_and_block(|scope| {
                for input in inputs {
                    scope.spawn(async {
                        // NOTE "try blocks" and "async closures" are both unstable features. As
                        // such, to report errors when they happen -- rather than collated at the
                        // end -- we have to resort to this awkward dance, so we can benefit from
                        // `?` syntax sugar. Rewrite with a "try block" once the feature is stable.
                        let result: CLIResult<()> = match input {
                            Ok(input) => {
                                // FIXME The cache is performing suboptimally; see `language.rs`
                                let lang_def = match cache.fetch(&input).await {
                                    Ok(lang_def) => lang_def,
                                    Err(error) => return Err(error),
                                };

                                tryvial::try_block! {
                                    let output = OutputFile::try_from(&input)?;

                                    log::info!(
                                        "Formatting {}, as {} using {}, to {}",
                                        input.source(),
                                        input.language(),
                                        input.query().to_string_lossy(),
                                        output
                                    );

                                    let mut buf_input = BufReader::new(input);
                                    let mut buf_output = BufWriter::new(output);

                                    formatter(
                                        &mut buf_input,
                                        &mut buf_output,
                                        &lang_def.query,
                                        &lang_def.language,
                                        &lang_def.grammar,
                                        Operation::Format {
                                            skip_idempotence,
                                            tolerate_parsing_errors,
                                        },
                                    )?;

                                    buf_output.into_inner()?.persist()?;
                                }
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

        Commands::Vis { format, input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            let input = Inputs::new(&config, &input).next().unwrap()?;
            let output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let lang_def = input.to_language_definition().await?;

            log::info!(
                "Visualising {}, as {}, to {}",
                input.source(),
                input.language(),
                output
            );

            let mut buf_input = BufReader::new(input);
            let mut buf_output = BufWriter::new(output);

            formatter(
                &mut buf_input,
                &mut buf_output,
                &lang_def.query,
                &lang_def.language,
                &lang_def.grammar,
                Operation::Visualise {
                    output_format: format.into(),
                },
            )?;
        }

        Commands::Cfg => {
            // Output collated configuration as TOML, with annotations about how we got there
            print!("{annotations}\n{config}");
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
