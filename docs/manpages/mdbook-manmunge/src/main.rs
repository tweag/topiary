mod error;
mod postprocess;
mod preprocess;
mod verbatim;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;

use postprocess::postprocess;
use preprocess::preprocess;

// Supported renderers
const SUPPORTED: &[&str] = &["man"];

#[derive(Debug, Parser)]
#[command(
    about,
    after_help = "Omit the COMMAND to access the mdBook interface for pre-processing",
    author,
    long_about = None,
    version,
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// mdBook interface for assessing pre-processor support
    Supports {
        /// Name of the renderer being used by mdBook
        renderer: String,
    },

    /// Post-process the rendered manpage from standard input to standard output
    PostProcess,
}

fn main() -> ExitCode {
    init_logger();
    let args = Args::parse();

    match args.command {
        None => {
            if let Err(error) = preprocess() {
                log::error!("Pre-processing failed: {error}");
                return ExitCode::FAILURE;
            }
        }

        Some(Commands::Supports { renderer }) => {
            if !SUPPORTED.iter().any(|&supported| renderer == supported) {
                log::error!("Unsupported pre-processor: {renderer}");
                log::info!("Supported pre-processors: {}", SUPPORTED.join(" "));
                return ExitCode::FAILURE;
            }
        }

        Some(Commands::PostProcess) => {
            if let Err(error) = postprocess() {
                log::error!("Post-processing failed: {error}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

fn init_logger() {
    let mut builder = Builder::new();

    if let Ok(var) = std::env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
    }

    builder.init();
}
