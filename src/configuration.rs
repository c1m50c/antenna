use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AntennaConfiguration {
    pub queries: Vec<AntennaQuery>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AntennaQuery {
    pub name: String,
    pub include: String,
    pub query: String,
    pub output: Option<HashSet<AntennaOutputMode>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AntennaOutputMode {
    Json {
        #[serde(default)]
        require_matches: bool,
        path: String,
    },

    Csv {
        path: String,
    },

    Occurrences,
}
