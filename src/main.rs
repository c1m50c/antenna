use std::fs;

use antenna::{configuration::AntennaConfiguration, AntennaResult};

fn main() -> AntennaResult<()> {
    let configuration = fs::read_to_string("./antenna.yml")?;
    let configuration = serde_yaml::from_str::<AntennaConfiguration>(&configuration)?;

    configuration
        .queries
        .iter()
        .try_for_each(|x| x.run(rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap()))?;

    Ok(())
}
