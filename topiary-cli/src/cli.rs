//! Command line interface argument parsing.

use clap::{ArgAction, ArgGroup, Args, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells::Shell};
use std::{io::stdout, path::PathBuf};

use log::LevelFilter;
use topiary::SupportedLanguage;

use crate::{
    configuration,
    error::{CLIResult, TopiaryError},
    visualisation,
};

#[derive(Debug, Parser)]
// NOTE Don't use infer_subcommands, as that could fossilise the interface. We define explicit
// aliases instead. (See https://clig.dev/#future-proofing)
#[command(about, author, long_about = None, version)]
pub struct Cli {
    // Global options
    #[command(flatten)]
    pub global: GlobalArgs,

    // Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

// These are "true" global arguments that are relevant to all subcommands
// NOTE Global arguments must be optional, even when defaults are specified
#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Configuration file
    #[arg(
        short = 'C',
        long,
        display_order = 100,
        env = "TOPIARY_CONFIG_FILE",
        global = true,
        hide_env_values = true
    )]
    pub configuration: Option<PathBuf>,

    /// Configuration collation mode
    #[arg(
        long,
        default_value = "merge",
        display_order = 101,
        env = "TOPIARY_CONFIG_COLLATION",
        global = true,
        hide_env_values = true
    )]
    pub configuration_collation: Option<configuration::CollationMode>,

    /// Logging verbosity (increased per occurrence)
    #[arg(
        short,
        long,
        action = ArgAction::Count,
        global = true,
        display_order = 102
    )]
    pub verbose: u8,
}

// NOTE This abstraction is largely to workaround clap-rs/clap#4707
#[derive(Args, Debug)]
pub struct FromStdin {
    /// Topiary supported language (for formatting stdin)
    #[arg(short, long)]
    pub language: SupportedLanguage,

    /// Topiary query file override (when formatting stdin)
    #[arg(short, long, requires = "language")]
    pub query: Option<PathBuf>,
}

// Subtype for exactly one input:
// * FILE       => Read input from disk, visualisation output to stdout
// * --language => Read input from stdin, visualisation output to stdout
#[derive(Args, Debug)]
#[command(
    // Require exactly one of --language, or FILES...
    group = ArgGroup::new("source")
        .multiple(false)
        .required(true)
        .args(&["language", "file"])
)]
pub struct ExactlyOneInput {
    #[command(flatten)]
    pub stdin: Option<FromStdin>,

    /// Input file (omit to read from stdin)
    ///
    /// Language detection and query selection is automatic, mapped from file extensions defined in
    /// the Topiary configuration.
    pub file: Option<PathBuf>,
}

// Subtype for at least one input
// * FILES...   => Read input(s) from disk, format in place
// * --language => Read input from stdin, output to stdout
#[derive(Args, Debug)]
#[command(
    // Require exactly one of --language, --query, or FILES...
    group = ArgGroup::new("source")
        .multiple(false)
        .required(true)
        .args(&["language", "files"])
)]
pub struct AtLeastOneInput {
    #[command(flatten)]
    pub stdin: Option<FromStdin>,

    /// Input files and directories (omit to read from stdin)
    ///
    /// Language detection and query selection is automatic, mapped from file extensions defined in
    /// the Topiary configuration.
    pub files: Vec<PathBuf>,
}

// NOTE When changing the subcommands, please update verify-documented-usage.sh respectively.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Format inputs
    #[command(alias = "fmt", display_order = 1)]
    Format {
        /// Consume as much as possible in the presence of parsing errors
        #[arg(short, long)]
        tolerate_parsing_errors: bool,

        /// Do not check that formatting twice gives the same output
        #[arg(short, long)]
        skip_idempotence: bool,

        #[command(flatten)]
        inputs: AtLeastOneInput,
    },

    /// Visualise the input's Tree-sitter parse tree
    #[command(aliases = &["vis", "visualize", "view"], display_order = 2)]
    Visualise {
        /// Visualisation format
        #[arg(short, long, default_value = "dot")]
        format: visualisation::Format,

        #[command(flatten)]
        input: ExactlyOneInput,
    },

    /// Print the current configuration
    #[command(alias = "cfg", display_order = 3)]
    Config,

    /// Generate shell completion script
    #[command(display_order = 100)]
    Completion {
        /// Shell (omit to detect from the environment)
        shell: Option<Shell>,
    },
}

/// Given a vector of paths, recursively expand those that identify as directories, in place
fn traverse_fs(files: &mut Vec<PathBuf>) -> CLIResult<()> {
    let mut expanded = vec![];

    for file in &mut *files {
        if file.is_dir() {
            let mut subfiles = file.read_dir()?.flatten().map(|f| f.path()).collect();
            traverse_fs(&mut subfiles)?;
            expanded.append(&mut subfiles);
        } else {
            expanded.push(file.to_path_buf());
        }
    }

    *files = expanded;
    Ok(())
}

/// Parse CLI arguments and normalise them for the caller
pub fn get_args() -> CLIResult<Cli> {
    let mut args = Cli::parse();

    // This is the earliest point that we can initialise the logger, from the --verbose flags,
    // before any fallible operations have started
    env_logger::Builder::new()
        .filter_level(match args.global.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    // NOTE We do not check that input files are actual files (with Path::is_file), because that
    // would break in the case of, for example, named pipes; thus also adding a platform dimension
    // to the check, which is simply not worth the complexity. We _could_ check by opening each
    // file, but that's going to be done sooner-or-later by Topiary, so there's no need.

    match &mut args.command {
        Commands::Format {
            inputs: AtLeastOneInput { files, .. },
            ..
        } => {
            // If we're given a list of FILES... then we assume them to all be on disk, even if "-"
            // is passed as an argument (i.e., interpret this as a valid filename, rather than as
            // stdin). We deduplicate this list to avoid formatting the same file multiple times
            // and recursively expand directories until we're left with a list of unique
            // (potential) files as input sources.
            files.sort_unstable();
            files.dedup();
            traverse_fs(files)?;
        }

        Commands::Visualise {
            input: ExactlyOneInput {
                file: Some(file), ..
            },
            ..
        } => {
            // Make sure our FILE is not a directory
            if file.is_dir() {
                return Err(TopiaryError::Bin(
                    format!("Cannot visualise directory \"{}\"; please provide a single file from disk or stdin.", file.to_string_lossy()),
                    None,
                ));
            }
        }

        // Attempt to detect shell from environment, when omitted
        Commands::Completion { shell: None } => {
            let detected_shell = Shell::from_env().ok_or(TopiaryError::Bin(
                "Cannot detect shell from environment".into(),
                None,
            ))?;

            args.command = Commands::Completion {
                shell: Some(detected_shell),
            };
        }

        _ => {}
    }

    Ok(args)
}

/// Generate shell completion script, for the given shell, and output to stdout
pub fn completion(shell: Shell) {
    generate(shell, &mut Cli::command(), "topiary", &mut stdout());
}
