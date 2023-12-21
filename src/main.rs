use std::fs;

use antenna::{process::index::Indexer, AntennaResult};
use args::AntennaArguments;
use clap::Parser;

mod args;

fn main() -> AntennaResult<()> {
    let AntennaArguments {
        configuration_file: settings_file,
    } = AntennaArguments::parse();

    let configuration_file = fs::read_to_string(settings_file)?;
    let configuration = serde_yaml::from_str(&configuration_file)?;

    let _indexer = Indexer::default().index(&configuration)?;

    Ok(())
}
