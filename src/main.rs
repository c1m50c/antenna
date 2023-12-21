use std::fs;

use antenna::{configuration::OutputMode, process::index::Indexer, AntennaResult};
use args::AntennaArguments;
use clap::Parser;
use tree_sitter::{Query, QueryCursor};

mod args;

fn main() -> AntennaResult<()> {
    let AntennaArguments {
        configuration_file: settings_file,
    } = AntennaArguments::parse();

    let configuration_file = fs::read_to_string(settings_file)?;
    let configuration = serde_yaml::from_str(&configuration_file)?;

    let indexer = Indexer::default().index(&configuration)?;

    for query in configuration.queries {
        if let Some(output_modes) = query.output {
            for output_mode in output_modes {
                match output_mode {
                    OutputMode::Occurrences => {
                        println!("{}", &query.name);

                        let files = indexer.get_files_by_query_name(&query.name).expect(
                            "The `Indexer` should contain indicies for the given query",
                        );

                        for file in files {
                            let query = Query::new(
                                file.recognized_language.as_tree_sitter_language(),
                                &query.query,
                            )?;

                            let mut query_cursor = QueryCursor::new();

                            let matches = query_cursor.matches(
                                &query,
                                file.tree.root_node(),
                                &file.content[..],
                            );

                            println!("> {:?} = `{}`", file.path, matches.count());
                        }
                    },
                }
            }
        }
    }

    Ok(())
}
