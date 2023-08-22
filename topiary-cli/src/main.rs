mod cli;
mod configuration;
mod error;
mod io;
mod language;
mod visualisation;

use std::{error::Error, process::ExitCode};

use crate::{
    cli::Commands,
    error::CLIResult,
    io::{Inputs, OutputFile},
    //language::LanguageDefinitionCache,
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
    env_logger::init();

    let args = cli::get_args()?;
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
            //let cache = LanguageDefinitionCache::new();

            let (_, tasks) = async_scoped::TokioScope::scope_and_block(|scope| {
                for input in inputs {
                    scope.spawn({
                        //let cache = &cache;

                        async {
                            match input {
                                Ok(mut input) => {
                                    // TODO BufReader and BufWriter
                                    let mut output = OutputFile::try_from(&input)?;

                                    //let lang_def = cache.fetch(&input).await?;
                                    let lang_def = input.to_language_definition().await?;

                                    log::info!(
                                        "Formatting {}, as {} using {}, to {}",
                                        input.source(),
                                        input.language(),
                                        input.query().to_string_lossy(),
                                        output
                                    );

                                    formatter(
                                        &mut input,
                                        &mut output,
                                        &lang_def.query,
                                        &lang_def.language,
                                        &lang_def.grammar,
                                        Operation::Format {
                                            skip_idempotence,
                                            tolerate_parsing_errors,
                                        },
                                    )?;

                                    output.persist()?;
                                }

                                Err(error) => {
                                    // By this point, we've lost any reference to the original
                                    // input; we trust that it is embedded into `error`.
                                    log::warn!("Skipping: {error}");
                                }
                            }

                            CLIResult::Ok(())
                        }
                    });
                }
            });

            // TODO This outputs all the errors from the concurrent tasks _after_ they've
            // completed. For join failures, that makes sense, but for Topiary errors, we lose
            // context in the logs. Topiary error handling should be done within the task.
            for task in tasks {
                match task {
                    Err(join_error) => print_error(&join_error),
                    Ok(Err(topiary_error)) => print_error(&topiary_error),
                    _ => {}
                }
            }
        }

        Commands::Vis { format, input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            // TODO BufReader and BufWriter
            let mut input = Inputs::new(&config, &input).next().unwrap()?;
            let mut output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let lang_def = input.to_language_definition().await?;

            log::info!(
                "Visualising {}, as {}, to {}",
                input.source(),
                input.language(),
                output
            );

            formatter(
                &mut input,
                &mut output,
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
