use std::{fs::OpenOptions, io::prelude::*, path::PathBuf};

use rayon::{prelude::*, ThreadPool};
use serde::{Deserialize, Serialize};
use tree_sitter::{Parser, Query, QueryCursor};

use crate::{AntennaError, AntennaResult, RecognizedLanguage};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AntennaConfiguration {
    pub queries: Vec<AntennaQuery>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AntennaQuery {
    pub name: String,
    pub include: String,
    pub query: String,
}

impl AntennaQuery {
    pub fn run(&self, thread_pool: ThreadPool) -> AntennaResult<()> {
        let paths = glob::glob(&self.include)?.collect::<Result<Vec<_>, _>>()?;

        fn read_file(path: &PathBuf) -> AntennaResult<(String, String)> {
            let extension = path
                .extension()
                .and_then(|x| x.to_ascii_uppercase().into_string().ok())
                .expect("File extension should be `Some`");

            let mut file = OpenOptions::new().read(true).open(path)?;
            let mut buffer = Vec::with_capacity(0xF4240);
            file.read_to_end(&mut buffer)?;

            let contents = String::from_utf8(buffer)?;
            Ok((contents, extension))
        }

        let query = |source_code: &str, extension: &str| -> AntennaResult<usize> {
            let language = RecognizedLanguage::from_language_extension(extension)
                .ok_or(AntennaError::Antenna {
                message: format!(
                    "Failed to find a `RecognizedLanguage` from the `{extension}` extension"
                ),
            })?;

            let query = Query::new(language.as_tree_sitter_language(), &self.query)?;
            let mut parser = Parser::new();

            parser.set_language(language.as_tree_sitter_language())?;

            let tree = parser
                .parse(source_code, None)
                .ok_or(AntennaError::Antenna {
                    message: "Failed to parse `source_code` into a `Tree`".to_string(),
                })?;

            let mut cursor = QueryCursor::new();

            let x = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

            Ok(x.count())
        };

        let query_results = thread_pool.install(move || -> Vec<AntennaResult<usize>> {
            paths
                .par_iter()
                .filter(|x| x.is_file())
                .flat_map(read_file)
                .map(|(contents, extension)| query(&contents, &extension))
                .collect::<Vec<_>>()
        });

        println!("{query_results:?}");

        Ok(())
    }
}
