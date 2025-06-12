use std::path::{Path, PathBuf};

use orfail::OrFail;

#[derive(Debug)]
pub struct IndexFile {
    pub path: PathBuf,
}

impl IndexFile {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            Self::load(path).or_fail()
        } else {
            Ok(Self { path })
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        path.exists().or_fail()?;
        Ok(Self { path })
    }

    pub fn repositories(&self) -> impl '_ + Iterator<Item = orfail::Result<RepositoryEntry>> {
        std::iter::empty() // TODO
    }
}

#[derive(Debug, Clone)]
pub enum IndexFileEntry {
    Repository,
    Chunk,
}

#[derive(Debug, Clone)]
pub struct RepositoryEntry {
    pub path: PathBuf,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub struct ChunkEntry {
    pub path: PathBuf,
    pub line: usize,
    pub embedding: Vec<f64>,
}
