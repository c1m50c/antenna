use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, help_template = "{name} ({version})\n{about-section}{author-section}\n{usage-heading} {usage}\n\n{all-args}")]
pub struct AntennaArguments {
    /// Path to the `antenna` configuration file.
    #[arg(
        short,
        long,
        env = "ANTENNA_CONFIGURATION_FILE",
        default_value = "./antenna.yml"
    )]
    pub configuration_file: PathBuf,

    /// Path to the `git` repository to analyze.
    #[arg(short, long, env = "ANTENNA_REPOSITORY", default_value = ".")]
    pub repository: PathBuf,
}
