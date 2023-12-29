use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::AntennaResult;

#[derive(Debug)]
pub struct TimeTraverser {
    repository: TimeTraverserRepository,
    commits: Vec<git2::Oid>,
}

impl TimeTraverser {
    pub fn new<P>(repository_path: P) -> AntennaResult<Self>
    where
        P: AsRef<Path>,
    {
        let repository = git2::Repository::open(&repository_path)?;
        let mut revwalk = repository.revwalk()?;

        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        revwalk.push_head()?;

        let commits = revwalk.collect::<Result<Vec<_>, _>>()?;

        let time_traverser = Self {
            repository: TimeTraverserRepository(
                repository_path.as_ref().to_path_buf(),
                repository,
            ),
            commits,
        };

        Ok(time_traverser)
    }
}

struct TimeTraverserRepository(PathBuf, git2::Repository);

impl fmt::Debug for TimeTraverserRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TimeTraverserRepository")
            .field(&self.0)
            .finish()
    }
}
