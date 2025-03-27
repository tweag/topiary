mod preprocess;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

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
    let args = Args::parse();

    match args.command {
        None => {
            if let Err(error) = preprocess() {
                eprintln!("Pre-processing failed: {error}");
                return ExitCode::FAILURE;
            }
        }

        Some(Commands::Supports { renderer }) => {
            if !SUPPORTED.iter().any(|&supported| renderer == supported) {
                eprintln!("Unsupported pre-processor: {renderer}");
                eprintln!("Supported pre-processors: {}", SUPPORTED.join(" "));
                return ExitCode::FAILURE;
            }
        }

        Some(Commands::PostProcess) => {
            // TODO
            println!("Post-process");
        }
    }

    ExitCode::SUCCESS
}
