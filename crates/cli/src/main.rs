use std::{fs, io::Write};

use antenna::{configuration::AntennaOutputMode, process::index::Indexer, AntennaResult};
use args::AntennaArguments;
use clap::Parser;

mod args;

fn main() -> AntennaResult<()> {
    let AntennaArguments {
        configuration_file: settings_file,
    } = AntennaArguments::parse();

    let configuration_file = fs::read_to_string(settings_file)?;
    let configuration = serde_yaml::from_str(&configuration_file)?;

    let indexer = Indexer::default().index(&configuration)?;

    for antenna_query in configuration.queries {
        let out_queries = antenna::process::execute_antenna_query(&antenna_query, &indexer)?;

        if let Some(output_modes) = &antenna_query.output {
            for output_mode in output_modes {
                match output_mode {
                    AntennaOutputMode::Occurrences => {
                        println!("{}", &antenna_query.name);

                        for out_query in &out_queries {
                            println!("> {:?} = `{}`", out_query.path, out_query.matches.len());
                        }
                    },

                    AntennaOutputMode::Json {
                        path,
                        require_matches,
                    } => {
                        let mut file =
                            fs::OpenOptions::new().create(true).write(true).open(path)?;

                        let json = match require_matches {
                            false => serde_json::to_string_pretty(&out_queries)?,
                            true => {
                                let out_queries = out_queries
                                    .split(|x| x.matches.is_empty())
                                    .flatten()
                                    .collect::<Vec<_>>();

                                serde_json::to_string_pretty(&out_queries)?
                            },
                        };

                        file.write_all(json.as_bytes())?;
                    },

                    AntennaOutputMode::Csv { path } => {
                        let file =
                            fs::OpenOptions::new().create(true).write(true).open(path)?;

                        let mut csv_writer = csv::Writer::from_writer(file);

                        for out_query in &out_queries {
                            for out_match in &out_query.matches {
                                let rows = antenna::out::csv::Capture::from_out_captures(
                                    &antenna_query.name,
                                    &out_query.path,
                                    &out_match.captures,
                                );

                                csv_writer.serialize(rows)?;
                            }
                        }

                        csv_writer.flush()?;
                    },
                }
            }
        }
    }

    Ok(())
}
