use std::{borrow::Cow, collections::HashMap};

use tree_sitter::{Query, QueryCursor};

use crate::{configuration::AntennaQuery, AntennaResult};

pub mod index;
pub mod time;

pub fn execute_antenna_query<'a>(
    antenna_query: &'a AntennaQuery,
    indexer: &'a index::Indexer,
) -> AntennaResult<Vec<crate::out::Query<'a>>> {
    let files = indexer
        .get_files_by_query_name(&antenna_query.name)
        .expect("The `Indexer` should contain indicies for the given query")
        .collect::<Vec<_>>();

    let mut out_queries = Vec::new();

    for file in &files {
        let mut out_query = crate::out::Query {
            name: Cow::Borrowed(&antenna_query.name),
            path: Cow::Borrowed(&file.path),
            matches: Vec::new(),
        };

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

        let query_matches =
            query_cursor.matches(&query, file.tree.root_node(), file.content.as_slice());

        for query_match in query_matches {
            let mut out_match = crate::out::Match {
                captures: Vec::new(),
            };

            let filtered = query_match
                .captures
                .iter()
                .filter(|x| capture_indices_to_names.contains_key(&x.index));

            let file_bytes = file.content.as_slice();

            for query_capture in filtered {
                let range = query_capture.node.range();

                let out_capture = crate::out::Capture {
                    name: capture_indices_to_names
                        .get(&query_capture.index)
                        .map(|&x| x.clone())
                        .unwrap_or_default(),

                    text: String::from_utf8(
                        file_bytes[range.start_byte..range.end_byte].to_vec(),
                    )?,

                    start_column: range.start_point.column,
                    start_line: range.start_point.row,
                    end_column: range.end_point.column,
                    end_line: range.end_point.row,
                };

                out_match.captures.push(out_capture);
            }

            out_query.matches.push(out_match);
        }

        out_queries.push(out_query);
    }

    Ok(out_queries)
}
