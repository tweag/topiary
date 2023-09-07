use clap::ValueEnum;
use topiary::Visualisation;

/// Visualisation output formats for Tree-sitter parse trees
// NOTE While redundant, we cannot implement clap::ValueEnum for topiary::Visualisation without
// breaking the orphan rules. So we have to maintain a local copy for the sake of the CLI.
#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    /// GraphViz DOT serialisation
    Dot,

    /// JSON serialisation
    Json,
}

impl From<Format> for Visualisation {
    fn from(visualisation: Format) -> Self {
        match visualisation {
            Format::Dot => Self::GraphViz,
            Format::Json => Self::Json,
        }
    }
}
