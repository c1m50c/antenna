use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CsvRow {
    pub match_idx: usize,
    pub query: String,
    pub path: String,
    pub capture: String,
    pub text: String,
    pub start_column: usize,
    pub start_line: usize,
    pub end_column: usize,
    pub end_line: usize,
}
