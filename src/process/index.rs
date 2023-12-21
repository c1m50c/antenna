use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    hash::Hash,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::Arc,
};

use rayon::prelude::*;
use tree_sitter::{Parser, Query, Tree};

use crate::{
    configuration::{AntennaConfiguration, AntennaQuery},
    AntennaError, AntennaResult, RecognizedLanguage,
};

#[derive(Debug, Default)]
pub struct Indexer {
    // TODO: The choice of `Arc` here is probably naive, it simply needs to be a reference to a value in `queries`. Optimize this.
    queries_by_name_and_language: HashMap<(String, RecognizedLanguage), Arc<Query>>,

    // TODO: The choice of `Arc` here is probably naive, it simply needs to be a reference to a value in `files`. Optimize this.
    files_by_query_name: HashMap<String, Arc<IndexedFile>>,

    // TODO: The choice of `Arc` here is probably naive, it simply needs to be a reference to a value in `files`. Optimize this.
    files_by_path: HashMap<PathBuf, Arc<IndexedFile>>,

    queries: HashMap<(RecognizedLanguage, String), Arc<Query>>,
    files: HashSet<Arc<IndexedFile>>,
}

impl Indexer {
    /// Consumes the [`Indexer`], creating indicies for the `queries` and `files` fields.
    pub fn index(self, configuration: &AntennaConfiguration) -> AntennaResult<Self> {
        let mut queries_by_name_and_language = self.queries_by_name_and_language;
        let mut files_by_query_name = self.files_by_query_name;
        let mut files_by_path = self.files_by_path;

        let mut queries = self.queries;
        let mut files = self.files;
        let mut errors = Vec::new();

        let indices = configuration
            .queries
            .par_iter()
            .map(Self::map_antenna_queries)
            .collect::<Vec<_>>();

        for index in indices {
            match index {
                Ok((indexed_queries, indexed_files, associated_query)) => {
                    files.extend(indexed_files);
                    queries.extend(indexed_queries);

                    files.iter().for_each(|x| {
                        files_by_query_name.insert(associated_query.to_owned(), Arc::clone(x));
                        files_by_path.insert(x.path.to_owned(), Arc::clone(x));
                    });

                    queries.iter().for_each(|((language, _), x)| {
                        queries_by_name_and_language
                            .insert((associated_query.to_owned(), *language), Arc::clone(x));
                    });
                },

                Err(err) => {
                    errors.push(err);
                },
            }
        }

        let constructed = Self {
            queries_by_name_and_language,
            files_by_query_name,
            files_by_path,
            files,
            queries,
        };

        Ok(constructed)
    }

    /// Retrieves an indexed [`Query`] via the associated `name` and `language`.
    pub fn get_query_by_name_and_language<S>(
        &self,
        name: String,
        language: RecognizedLanguage,
    ) -> Option<&Query> {
        self.queries_by_name_and_language
            .get(&(name, language))
            .map(|x| x.as_ref())
    }

    /// Retrieves an [`IndexedFile`] via the associated `query_name`.
    pub fn get_file_by_query_name<S>(&self, query_name: S) -> Option<&IndexedFile>
    where
        S: AsRef<str>,
    {
        self.files_by_query_name
            .get(query_name.as_ref())
            .map(|x| x.as_ref())
    }

    /// Retrieves an [`IndexedFile`] via the associated `path`.
    pub fn get_file_by_path<P>(&self, path: P) -> Option<&IndexedFile>
    where
        P: AsRef<Path>,
    {
        self.files_by_path.get(path.as_ref()).map(|x| x.as_ref())
    }

    /// Retrieves an [`Iterator`] of references to all [`Queries`](Query) indexed in the [`Indexer`].
    pub fn queries(&self) -> impl Iterator<Item = &Query> {
        self.queries.values().map(|x| x.as_ref())
    }

    /// Retrieves an [`Iterator`] of references to all [`IndexedFiles`](IndexedFile) indexed in the [`Indexer`].
    pub fn files(&self) -> impl Iterator<Item = &IndexedFile> {
        self.files.iter().map(|x| x.as_ref())
    }
}

impl Indexer {
    #[allow(clippy::type_complexity)]
    fn map_antenna_queries(
        antenna_query: &AntennaQuery,
    ) -> AntennaResult<(
        HashMap<(RecognizedLanguage, String), Arc<Query>>,
        HashSet<Arc<IndexedFile>>,
        String,
    )> {
        let include_paths = glob::glob(&antenna_query.include)?;
        let mut queries = HashMap::new();
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

        let unique_languages = files
            .iter()
            .map(|x| x.recognized_language)
            .collect::<HashSet<_>>();

        for language in unique_languages {
            match Query::new(language.as_tree_sitter_language(), &antenna_query.query) {
                Ok(query) => {
                    queries.insert((language, antenna_query.query.clone()), Arc::new(query));
                },

                Err(err) => errors.push(AntennaError::Query { inner: err }),
            }
        }

        if !errors.is_empty() {
            return Err(AntennaError::Collection { errors });
        }

        Ok((queries, files, antenna_query.name.clone()))
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
