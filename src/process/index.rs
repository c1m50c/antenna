use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::prelude::*,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use tree_sitter::{Parser, Query, Tree};

use crate::{
    configuration::{AntennaConfiguration, AntennaQuery},
    AntennaError, AntennaResult, RecognizedLanguage,
};

#[derive(Debug, Default)]
pub struct Indexer {
    queries: HashMap<(String, RecognizedLanguage), Query>,
    files: HashMap<PathBuf, IndexedFile>,
}

impl Indexer {
    /// Consumes the [`Indexer`], creating indicies for the `queries` and `files` fields.
    pub fn index(self, configuration: &AntennaConfiguration) -> AntennaResult<Self> {
        let mut queries = self.queries;
        let mut errors = Vec::<AntennaError>::new();
        let mut files = self.files;

        let includes = configuration.queries.iter().map(|x| x.include.as_str());
        let (file_paths, glob_errors) = Self::read_includes(includes);

        if !glob_errors.is_empty() {
            return Err(AntennaError::Collection {
                errors: glob_errors,
            });
        }

        let indexed_files = file_paths
            .into_par_iter()
            .map(|x| (Self::index_file(&x), x))
            .collect::<Vec<_>>();

        for (indexed_file, path) in indexed_files {
            match indexed_file {
                Ok(indexed_file) => {
                    files.insert(path, indexed_file);
                },

                Err(err) => {
                    errors.push(err);
                },
            }
        }

        let indexed_queries = configuration
            .queries
            .par_iter()
            .map(|x| Self::index_query(x, &files))
            .collect::<Vec<_>>();

        for indexed_query in indexed_queries {
            match indexed_query {
                Ok(indexed_query) => {
                    for (key, value) in indexed_query {
                        queries.insert(key, value);
                    }
                },

                Err(err) => {
                    errors.push(err);
                },
            }
        }

        if !errors.is_empty() {
            return Err(AntennaError::Collection { errors });
        }

        Ok(Self { queries, files })
    }

    /// Attempts to return an [`IndexedFile`] contained in the [`Indexer`].
    #[inline(always)]
    pub fn get_file(&self, path: &Path) -> Option<&IndexedFile> { self.files.get(path) }

    /// Returns a reference to the `files` field.
    #[inline(always)]
    pub fn get_files(&self) -> &HashMap<PathBuf, IndexedFile> { &self.files }

    /// Attempts to return an [`Query`] contained in the [`Indexer`].
    #[inline(always)]
    pub fn get_query(&self, name: String, language: RecognizedLanguage) -> Option<&Query> {
        self.queries.get(&(name, language))
    }

    /// Returns a reference to the `queries` field.
    #[inline(always)]
    pub fn get_queries(&self) -> &HashMap<(String, RecognizedLanguage), Query> { &self.queries }
}

impl Indexer {
    /// Converts an iterator over the values of an [`AntennaConfiguration`]'s `include` field to a [`Paths`](glob::Paths) iterator.
    fn read_includes<'a, I>(include: I) -> (Vec<PathBuf>, Vec<AntennaError>)
    where
        I: Iterator<Item = &'a str>,
    {
        let mut errors = Vec::<AntennaError>::new();
        let mut oks = Vec::<PathBuf>::new();

        for paths in include.map(glob::glob) {
            match paths {
                Err(err) => errors.push(AntennaError::Pattern { inner: err }),
                Ok(paths) => {
                    for path in paths {
                        match path {
                            Err(err) => errors.push(AntennaError::Glob { inner: err }),
                            Ok(path) => oks.push(path),
                        }
                    }
                },
            }
        }

        (oks, errors)
    }

    fn index_query(
        query: &AntennaQuery,
        files: &HashMap<PathBuf, IndexedFile>,
    ) -> AntennaResult<HashMap<(String, RecognizedLanguage), Query>> {
        let includes = glob::glob(&query.include)?.flat_map(|x| x.ok());
        let mut queries = HashMap::new();

        for path in includes {
            let indexed_file = files.get(&path).ok_or(AntennaError::Antenna {
                message: format!(
                    "Included path `{:?}` not found within `Indexer.files`",
                    &path
                ),
            })?;

            queries.insert(
                (query.name.clone(), indexed_file.recognized_language),
                Query::new(
                    indexed_file.recognized_language.as_tree_sitter_language(),
                    &query.query,
                )?,
            );
        }

        Ok(queries)
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
