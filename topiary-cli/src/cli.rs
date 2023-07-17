use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, author, long_about = None, version)]
struct Cli {
    // Global options
    /// Configuration file
    // TODO Formerly --configuration-file and --configuration-override. Investigate...
    #[arg(
        short = 'C',
        long,
        env = "TOPIARY_CONFIG",
        global = true,
        display_order = 100
    )]
    configuration: Option<PathBuf>,

    /// Consume as much as possible in the presence of parsing errors
    #[arg(short, long, global = true, display_order = 101)]
    tolerate_parsing_errors: bool,

    // Subcommands
    // TODO No subcommand => "fmt", with all its options
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Format input files [default]
    #[command(
        alias = "format",
        group = ArgGroup::new("source")
            .multiple(false)
            .required(true)
            .args(&["language", "query", "files"]),
        display_order = 1
    )]
    Fmt {
        /// Do not check that formatting twice gives the same output
        #[arg(short, long)]
        skip_idempotence: bool,

        /// Topiary supported language (for formatting stdin)
        // TODO Supported language enum
        #[arg(short, long)]
        language: Option<String>,

        /// Topiary query file (for formatting stdin)
        #[arg(short, long)]
        query: Option<PathBuf>,

        /// Input files and directories (omit to read from stdin)
        files: Vec<PathBuf>,
    },

    /// Visualise the input file's Tree-sitter parse tree
    #[command(
        aliases = &["visualise", "visualize", "view"],
        group = ArgGroup::new("source")
            .multiple(false)
            .required(true)
            .args(&["language", "query", "file"]),
        display_order = 2
    )]
    Vis {
        /// Visualisation format
        // TODO Supported visualisation format enum
        #[arg(short, long)]
        format: String,

        /// Topiary supported language (for formatting stdin)
        // TODO Supported language enum
        #[arg(short, long)]
        language: Option<String>,

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
