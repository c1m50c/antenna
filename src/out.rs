use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "query", tag = "type")]
pub struct Query {
    pub name: String,
    pub path: String,
    pub matches: Vec<Match>,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "match", tag = "type")]
pub struct Match {
    pub captures: Vec<Capture>,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "capture", tag = "type")]
pub struct Capture {
    pub name: String,
    pub text: String,
    pub start_column: usize,
    pub start_line: usize,
    pub end_column: usize,
    pub end_line: usize,
}

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
