//! Command line interface argument parsing.

use clap::{ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;
use topiary::SupportedLanguage;

use crate::{
    configuration,
    error::{CLIResult, TopiaryError},
    visualisation,
};

#[derive(Debug, Parser)]
// NOTE infer_subcommands would be useful, but our heavy use of aliases is problematic (see
// clap-rs/clap#5021)
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
        default_value = "coalesce",
        display_order = 101,
        env = "TOPIARY_CONFIG_COLLATION",
        global = true,
        hide_env_values = true,

        // FIXME There appears to be a bug with clap: If this argument is specified via its
        // environment variable, then the required argument (--configuration) *only* works if it is
        // also specified via its environment variable. If you use the CLI argument, it complains
        // that the argument doesn't exist. This behaviour only occurs with subcommands, but that
        // is exactly our use case, here. (See clap-rs/clap#5020)
        requires = "configuration"
    )]
    pub configuration_collation: Option<configuration::CollationMode>,
}

// These are "parser" global arguments; i.e., those that are relevant to all subcommands that will
// parse input. They will need to be added to all such subcommands, with #[command(flatten)].
#[derive(Args, Debug)]
pub struct ParseArgs {
    /// Consume as much as possible in the presence of parsing errors
    #[arg(short, long)]
    tolerate_parsing_errors: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Format inputs
    // NOTE FILES...             => Read input(s) from disk, format in place
    //      --language | --query => Read input from stdin, output to stdout
    #[command(
        alias = "format",
        display_order = 1,

        // Require exactly one of --language, --query, or FILES...
        group = ArgGroup::new("source")
            .multiple(false)
            .required(true)
            .args(&["language", "query", "files"])
    )]
    Fmt {
        #[command(flatten)]
        parse: ParseArgs,

        /// Do not check that formatting twice gives the same output
        #[arg(short, long)]
        skip_idempotence: bool,

        /// Topiary supported language (for formatting stdin)
        #[arg(short, long)]
        language: Option<SupportedLanguage>,

        /// Topiary query file (for formatting stdin)
        #[arg(short, long)]
        query: Option<PathBuf>,

        /// Input files and directories (omit to read from stdin)
        files: Vec<PathBuf>,
    },

    /// Visualise the input's Tree-sitter parse tree
    // NOTE FILE                 => Read input from disk, visualisation output to stdout
    //      --language | --query => Read input from stdin, visualisation output to stdout
    #[command(
        aliases = &["visualise", "visualize", "view"],
        display_order = 2,

        // Require exactly one of --language, --query, or FILE
        group = ArgGroup::new("source")
            .multiple(false)
            .required(true)
            .args(&["language", "query", "file"])
    )]
    Vis {
        #[command(flatten)]
        parse: ParseArgs,

        /// Visualisation format
        #[arg(short, long, default_value = "dot")]
        format: visualisation::Format,

        /// Topiary supported language (for formatting stdin)
        #[arg(short, long)]
        language: Option<SupportedLanguage>,

        /// Topiary query file (for formatting stdin)
        #[arg(short, long)]
        query: Option<PathBuf>,

        /// Input file (omit to read from stdin)
        file: Option<PathBuf>,
    },

    /// Print the current configuration
    #[command(alias = "config", display_order = 3)]
    Cfg,
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

    // NOTE We do not check that input files are actual files (with Path::is_file), because that
    // would break in the case of, for example, named pipes; thus also adding a platform dimension
    // to the check, which is simply not worth the complexity. We _could_ check by opening each
    // file, but that's going to be done sooner-or-later by Topiary, so there's no need.

    match &mut args.command {
        Commands::Fmt { files, .. } => {
            // If we're given a list of FILES... then we assume them to all be on disk, even if "-"
            // is passed as an argument (i.e., interpret this as a valid filename, rather than as
            // stdin). We deduplicate this list to avoid formatting the same file multiple times
            // and recursively expand directories until we're left with a list of unique
            // (potential) files as input sources.
            files.sort_unstable();
            files.dedup();
            traverse_fs(files)?;
        }

        Commands::Vis {
            file: Some(file), ..
        } => {
            // Make sure our FILE is not a directory
            if file.is_dir() {
                return Err(TopiaryError::Bin(
                    format!("Cannot visualise directory \"{}\"; please provide a single file from disk or stdin.", file.to_string_lossy()),
                    None,
                ));
            }
        }

        _ => {}
    }

    Ok(args)
}
