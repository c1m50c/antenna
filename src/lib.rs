use std::string::FromUtf8Error;

use glob::{GlobError, PatternError};
use thiserror::Error;
use tree_sitter::{Language, LanguageError, QueryError};

pub mod configuration;
pub mod process;

/// Wrapper type of a [`Result`] where the [`Err`] variant is a [`AntennaError`].
pub type AntennaResult<T> = Result<T, AntennaError>;

/// An [`Err`] returned by an `antenna` function.
#[derive(Debug, Error)]
pub enum AntennaError {
    #[error("tree sitter query error")]
    Query {
        #[from]
        inner: QueryError,
    },

    #[error("glob pattern error")]
    Pattern {
        #[from]
        inner: PatternError,
    },

    #[error("language error")]
    Language {
        #[from]
        inner: LanguageError,
    },

    #[error("glob error")]
    Glob {
        #[from]
        inner: GlobError,
    },

    #[error("io error")]
    Io {
        #[from]
        inner: std::io::Error,
    },

    #[error("from utf8 error")]
    FromUtf8 {
        #[from]
        inner: FromUtf8Error,
    },

    #[error("yaml error")]
    Yaml {
        #[from]
        inner: serde_yaml::Error,
    },

    #[error("antenna error")]
    Antenna { message: String },

    #[error("error collection")]
    Collection { errors: Vec<AntennaError> },
}

/// An enumerator holding variants that are languages _recognized_ by `antenna`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RecognizedLanguage {
    #[cfg(feature = "rust")]
    Rust,
}

impl RecognizedLanguage {
    /// Converts the [`RecognizedLanguage`] to a Tree Sitter [`Language`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use antenna::RecognizedLanguage;
    ///
    /// assert_eq!(
    ///     RecognizedLanguage::Rust.as_tree_sitter_language(),
    ///     tree_sitter_rust::language()
    /// );
    /// ```
    pub fn as_tree_sitter_language(&self) -> Language {
        match self {
            #[cfg(feature = "rust")]
            Self::Rust => tree_sitter_rust::language(),
        }
    }

    /// Attempts to construct a [`RecognizedLanguage`] from a language's file extension.
    /// If the extension is not mappable to a _[`RecognizedLanguage`]_, this function will return [`None`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use antenna::RecognizedLanguage;
    ///
    /// assert_eq!(
    ///     RecognizedLanguage::from_language_extension("rs"),
    ///     Some(RecognizedLanguage::Rust)
    /// );
    /// ```
    pub fn from_language_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            #[cfg(feature = "rust")]
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }
}
