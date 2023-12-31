use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    hash::Hash,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::Arc,
};

use rayon::prelude::*;
use tree_sitter::{Parser, Tree};

use crate::{
    configuration::{AntennaConfiguration, AntennaQuery},
    AntennaError, AntennaResult, RecognizedLanguage,
};

#[derive(Debug, Default)]
pub struct Indexer {
    // TODO: The choice of `Arc` here is probably naive, it simply needs to be a reference to a value in `files`. Optimize this.
    files_by_query_name: HashMap<String, HashSet<Arc<IndexedFile>>>,

    // TODO: The choice of `Arc` here is probably naive, it simply needs to be a reference to a value in `files`. Optimize this.
    files_by_path: HashMap<PathBuf, Arc<IndexedFile>>,

    files: HashSet<Arc<IndexedFile>>,
}

impl Indexer {
    /// Consumes the [`Indexer`], creating indicies for all [queries](Query) and [files](IndexedFile) found in the given [`configuration`](AntennaConfiguration).
    pub fn index(self, configuration: &AntennaConfiguration) -> AntennaResult<Self> {
        let mut files_by_query_name = self.files_by_query_name;
        let mut files_by_path = self.files_by_path;

        let mut files = self.files;
        let mut errors = Vec::new();

        let indices = configuration
            .queries
            .par_iter()
            .map(Self::map_antenna_queries)
            .collect::<Vec<_>>();

        for index in indices {
            match index {
                Ok((indexed_files, (associated_query, associated_include))) => {
                    files.extend(indexed_files);

                    let associated_glob =
                        glob::glob(&associated_include)?.collect::<Result<HashSet<_>, _>>()?;

                    files.iter().for_each(|x| {
                        if associated_glob.contains(&x.path) {
                            match files_by_query_name.get_mut(&associated_query) {
                                Some(collection) => {
                                    collection.insert(Arc::clone(x));
                                },

                                None => {
                                    files_by_query_name.insert(
                                        associated_query.to_owned(),
                                        HashSet::from_iter(vec![Arc::clone(x)]),
                                    );
                                },
                            }
                        }

                        files_by_path.insert(x.path.to_owned(), Arc::clone(x));
                    });
                },

                Err(err) => {
                    errors.push(err);
                },
            }
        }

        let constructed = Self {
            files_by_query_name,
            files_by_path,
            files,
        };

        Ok(constructed)
    }

    /// Retrieves an [`IndexedFile`] via the associated `query_name`.
    pub fn get_files_by_query_name<S>(
        &self,
        query_name: S,
    ) -> Option<impl Iterator<Item = &IndexedFile>>
    where
        S: AsRef<str>,
    {
        self.files_by_query_name
            .get(query_name.as_ref())
            .map(|x| x.iter().map(|x| x.as_ref()))
    }

    /// Retrieves an [`IndexedFile`] via the associated `path`.
    pub fn get_file_by_path<P>(&self, path: P) -> Option<&IndexedFile>
    where
        P: AsRef<Path>,
    {
        self.files_by_path.get(path.as_ref()).map(|x| x.as_ref())
    }

    /// Retrieves an [`Iterator`] of references to all [`IndexedFiles`](IndexedFile) indexed in the [`Indexer`].
    pub fn files(&self) -> impl Iterator<Item = &IndexedFile> {
        self.files.iter().map(|x| x.as_ref())
    }
}

impl Indexer {
    /// Maps an [`AntennaQuery`] to values for the `queries` and `files` fields in an [`Indexer`].
    #[allow(clippy::type_complexity)]
    fn map_antenna_queries(
        antenna_query: &AntennaQuery,
    ) -> AntennaResult<(HashSet<Arc<IndexedFile>>, (String, String))> {
        let include_paths = glob::glob(&antenna_query.include)?;
        let mut files = HashSet::new();
        let mut errors = Vec::new();

        for path in include_paths {
            match path {
                Ok(path) => {
                    match Self::index_file(path) {
                        Ok(indexed) => {
                            files.insert(Arc::new(indexed));
                        },

                        Err(err) => errors.push(err),
                    }
                },

                Err(err) => errors.push(AntennaError::Glob { inner: err }),
            }
        }

        if !errors.is_empty() {
            return Err(AntennaError::Collection { errors });
        }

        Ok((
            files,
            (antenna_query.name.clone(), antenna_query.include.clone()),
        ))
    }

    /// Creates an [`IndexedFile`] via reading the file at the given `path`.
    fn index_file<P>(path: P) -> AntennaResult<IndexedFile>
    where
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut content = Vec::with_capacity(0xF4240);
        let mut parser = Parser::new();

        file.read_to_end(&mut content)?;

        let extension = path
            .as_ref()
            .extension()
            .and_then(|x| x.to_os_string().into_string().ok())
            .ok_or(AntennaError::Antenna {
                message: format!("File `{:?}` does not have an extension", path.as_ref()),
            })?;

        let recognized_language = RecognizedLanguage::from_language_extension(&extension)
            .ok_or(AntennaError::Antenna {
                message: format!(
                    "File `{:?}` is not part of a `RecognizedLanguage`",
                    path.as_ref()
                ),
            })?;

        let name = path
            .as_ref()
            .file_name()
            .and_then(|x| x.to_os_string().into_string().ok())
            .ok_or(AntennaError::Antenna {
                message: format!("Could not retrieve `{:?}`'s file name", path.as_ref()),
            })?;

        parser.set_language(recognized_language.as_tree_sitter_language())?;

        let tree = parser
            .parse(String::from_utf8(content.clone())?, None)
            .ok_or(AntennaError::Antenna {
                message: format!("Failed to parse `{:?}`", path.as_ref()),
            })?;

        Ok(IndexedFile {
            path: path.as_ref().to_path_buf(),
            recognized_language,
            extension,
            content,
            name,
            tree,
        })
    }
}

/// Represents a file that has been indexed via an [`Indexer`].
#[derive(Debug)]
pub struct IndexedFile {
    pub recognized_language: RecognizedLanguage,
    pub extension: String,
    pub content: Vec<u8>,
    pub path: PathBuf,
    pub name: String,
    pub tree: Tree,
}

impl PartialEq for IndexedFile {
    fn eq(&self, other: &Self) -> bool {
        self.recognized_language == other.recognized_language
            && self.extension == other.extension
            && self.content == other.content
            && self.path == other.path
            && self.name == other.name
    }
}

impl Eq for IndexedFile {}

impl Hash for IndexedFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.recognized_language.hash(state);
        self.extension.hash(state);
        self.content.hash(state);
        self.path.hash(state);
        self.name.hash(state);
    }
}
