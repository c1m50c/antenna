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
}
