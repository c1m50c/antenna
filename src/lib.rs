use tree_sitter::Language;

/// An enumerator holding variants that are languages _recognized_ by `antenna`.
#[derive(Debug, PartialEq)]
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
        match extension {
            #[cfg(feature = "rust")]
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }
}
