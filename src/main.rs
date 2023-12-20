use std::fs;

use antenna::{configuration::AntennaConfiguration, AntennaResult};
use clap::Parser;

mod args;

fn main() -> AntennaResult<()> {
    let arguments = args::AntennaArguments::parse();

    let configuration = serde_yaml::from_str::<AntennaConfiguration>(&fs::read_to_string(
        arguments.settings_file,
    )?)?;

    configuration.queries.iter().try_for_each(|x| {
        x.run(
            rayon::ThreadPoolBuilder::new()
                .num_threads(4)
                .build()
                .unwrap(),
        )
    })?;

    Ok(())
}
