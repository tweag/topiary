use std::io;

use clap::ValueEnum;

use crate::error::CLIResult;
use topiary::Language;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Visualisation {
    Json,
}

pub fn visualiser(
    input: &mut dyn io::Read,
    output: &mut dyn io::Write,
    query: &mut dyn io::Read,
    language: Option<Language>,
    visualisation: Visualisation,
) -> CLIResult<()> {
    Ok(())
}
