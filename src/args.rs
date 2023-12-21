use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct AntennaArguments {
    /// Path to the `antenna` settings file.
    #[arg(
        short,
        long,
        env = "ANTENNA_SETTINGS_FILE",
        default_value = "./antenna.yml"
    )]
    pub configuration_file: PathBuf,
}
