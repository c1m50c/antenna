use std::{borrow::Cow, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "query", tag = "type")]
pub struct Query<'a> {
    pub name: Cow<'a, str>,
    pub path: Cow<'a, Path>,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "match", tag = "type")]
pub struct Match {
    pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", rename = "capture", tag = "type")]
pub struct Capture {
    pub text: String,
    pub name: String,
    pub start_column: usize,
    pub start_line: usize,
    pub end_column: usize,
    pub end_line: usize,
}

pub mod csv {
    use serde::{Deserialize, Serialize};
    use std::{borrow::Cow, path::Path};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Capture<'a> {
        pub index: usize,
        pub query: Cow<'a, str>,
        pub path: Cow<'a, Path>,
        pub capture: Cow<'a, str>,
        pub text: Cow<'a, str>,
        pub start_column: usize,
        pub start_line: usize,
        pub end_column: usize,
        pub end_line: usize,
    }

    impl<'a> Capture<'a> {
        pub fn from_out_captures(
            query: &'a str,
            path: &'a Path,
            out_captures: &'a [super::Capture],
        ) -> Vec<Self> {
            let out_csv_captures =
                out_captures
                    .into_iter()
                    .enumerate()
                    .map(|(index, capture)| {
                        Capture {
                            index,
                            query: Cow::Borrowed(query),
                            path: Cow::Borrowed(path),
                            capture: Cow::Borrowed(&capture.name),
                            text: Cow::Borrowed(&capture.text),
                            start_column: capture.start_column,
                            start_line: capture.start_line,
                            end_column: capture.end_column,
                            end_line: capture.end_line,
                        }
                    });

            out_csv_captures.collect()
        }
    }
}
