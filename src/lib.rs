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

    #[cfg(feature = "python")]
    Python,

    #[cfg(feature = "typescript")]
    TypeScript,

    #[cfg(feature = "typescript")]
    Tsx,

    #[cfg(feature = "javascript")]
    JavaScript,

    #[cfg(feature = "go")]
    Go,

    #[cfg(feature = "cpp")]
    Cpp,

    #[cfg(feature = "java")]
    Java,

    #[cfg(feature = "c")]
    C,

    #[cfg(feature = "ruby")]
    Ruby,

    #[cfg(feature = "html")]
    Html,

    #[cfg(feature = "css")]
    Css,

    #[cfg(feature = "swift")]
    Swift,

    #[cfg(feature = "c-sharp")]
    CSharp,

    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(feature = "yaml")]
    Yaml,
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

            #[cfg(feature = "python")]
            Self::Python => tree_sitter_python::language(),

            #[cfg(feature = "typescript")]
            Self::TypeScript => tree_sitter_typescript::language_typescript(),

            #[cfg(feature = "typescript")]
            Self::Tsx => tree_sitter_typescript::language_tsx(),

            #[cfg(feature = "javascript")]
            Self::JavaScript => tree_sitter_javascript::language(),

            #[cfg(feature = "go")]
            Self::Go => tree_sitter_go::language(),

            #[cfg(feature = "cpp")]
            Self::Cpp => tree_sitter_cpp::language(),

            #[cfg(feature = "java")]
            Self::Java => tree_sitter_java::language(),

            #[cfg(feature = "c")]
            Self::C => tree_sitter_c::language(),

            #[cfg(feature = "ruby")]
            Self::Ruby => tree_sitter_ruby::language(),

            #[cfg(feature = "html")]
            Self::Html => tree_sitter_html::language(),

            #[cfg(feature = "css")]
            Self::Css => tree_sitter_css::language(),

            #[cfg(feature = "swift")]
            Self::Swift => tree_sitter_swift::language(),

            #[cfg(feature = "c-sharp")]
            Self::CSharp => tree_sitter_c_sharp::language(),

            #[cfg(feature = "json")]
            Self::Json => tree_sitter_json::language(),

            #[cfg(feature = "toml")]
            Self::Toml => tree_sitter_toml::language(),

            #[cfg(feature = "yaml")]
            Self::Yaml => tree_sitter_yaml::language(),
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

            #[cfg(feature = "python")]
            "py" => Some(Self::Python),

            #[cfg(feature = "typescript")]
            "ts" => Some(Self::TypeScript),

            #[cfg(feature = "typescript")]
            "tsx" => Some(Self::Tsx),

            #[cfg(feature = "javascript")]
            "js" | "jsx" => Some(Self::JavaScript),

            #[cfg(feature = "go")]
            "go" => Some(Self::Go),

            #[cfg(feature = "cpp")]
            "cpp" | "cxx" => Some(Self::Cpp),

            #[cfg(feature = "java")]
            "java" => Some(Self::Java),

            #[cfg(feature = "c")]
            "c" => Some(Self::C),

            #[cfg(feature = "ruby")]
            "rb" => Some(Self::Ruby),

            #[cfg(feature = "html")]
            "html" => Some(Self::Html),

            #[cfg(feature = "css")]
            "css" => Some(Self::Css),

            #[cfg(feature = "swift")]
            "swift" => Some(Self::Swift),

            #[cfg(feature = "c-sharp")]
            "cs" => Some(Self::CSharp),

            #[cfg(feature = "json")]
            "json" => Some(Self::Json),

            #[cfg(feature = "toml")]
            "toml" => Some(Self::Toml),

            #[cfg(feature = "yaml")]
            "yaml" | "yml" => Some(Self::Yaml),

            _ => None,
        }
    }
}
