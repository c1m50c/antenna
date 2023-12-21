use std::fs;

use antenna::{configuration::AntennaConfiguration, process::index::Indexer, AntennaResult};
use clap::Parser;

mod args;

fn main() -> AntennaResult<()> {
    let arguments = args::AntennaArguments::parse();

    let configuration = serde_yaml::from_str::<AntennaConfiguration>(&fs::read_to_string(
        arguments.settings_file,
    )?)?;

    let _indexer = Indexer::default().index(&configuration)?;

    Ok(())
}
