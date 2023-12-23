use std::{collections::HashMap, fs};

use antenna::{configuration::OutputMode, csv::CsvRow, process::index::Indexer, AntennaResult};
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

    for antenna_query in configuration.queries {
        if let Some(output_modes) = antenna_query.output {
            let files = indexer
                .get_files_by_query_name(&antenna_query.name)
                .expect("The `Indexer` should contain indicies for the given query")
                .collect::<Vec<_>>();

            for output_mode in output_modes {
                match output_mode {
                    OutputMode::Occurrences => {
                        println!("{}", &antenna_query.name);

                        for file in &files {
                            let query = Query::new(
                                file.recognized_language.as_tree_sitter_language(),
                                &antenna_query.query,
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

                    OutputMode::Csv { path } => {
                        let file =
                            fs::OpenOptions::new().create(true).write(true).open(path)?;

                        let mut csv_writer = csv::Writer::from_writer(file);

                        for file in &files {
                            let query = Query::new(
                                file.recognized_language.as_tree_sitter_language(),
                                &antenna_query.query,
                            )?;

                            let mut query_cursor = QueryCursor::new();

                            let capture_indices_to_names = query
                                .capture_names()
                                .iter()
                                .flat_map(|x| query.capture_index_for_name(x).map(|i| (i, x)))
                                .collect::<HashMap<_, _>>();

                            let query_matches = query_cursor.matches(
                                &query,
                                file.tree.root_node(),
                                file.content.as_slice(),
                            );

                            for (query_match_idx, query_match) in query_matches.enumerate() {
                                let filtered = query_match.captures.iter().filter(|x| {
                                    capture_indices_to_names.contains_key(&x.index)
                                });

                                let file_bytes = file.content.as_slice();

                                for query_capture in filtered {
                                    let range = query_capture.node.range();

                                    let bytes = &file_bytes[range.start_byte..range.end_byte];

                                    let csv_row = CsvRow {
                                        capture: capture_indices_to_names
                                            .get(&query_capture.index)
                                            .map(|&x| x.clone())
                                            .unwrap_or_default(),

                                        text: String::from_utf8_lossy(bytes).to_string(),
                                        path: file.path.to_string_lossy().to_string(),
                                        start_column: range.start_point.column,
                                        query: antenna_query.name.to_owned(),
                                        end_column: range.end_point.column,
                                        start_line: range.start_point.row,
                                        end_line: range.end_point.row,
                                        match_idx: query_match_idx,
                                    };

                                    csv_writer.serialize(csv_row)?;
                                }
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
