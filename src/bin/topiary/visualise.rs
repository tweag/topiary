// This is somewhat redundant, but we cannot implement clap::ValueEnum for topiary::Visualisation
// without breaking the orphan rules. So we have to maintain a local copy for the sake of the CLI.

use clap::ValueEnum;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Visualisation {
    Dot,
    Json,
}

impl From<Visualisation> for topiary::Visualisation {
    fn from(visualisation: Visualisation) -> Self {
        match visualisation {
            Visualisation::Dot => Self::GraphViz,
            Visualisation::Json => Self::Json,
        }
    }
}
