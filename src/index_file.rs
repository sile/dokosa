use std::path::{Path, PathBuf};

use orfail::OrFail;

use crate::embedder::Embedding;

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

impl nojson::DisplayJson for RepositoryEntry {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("path", &self.path)?;
            f.member("commit", &self.commit)
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for RepositoryEntry {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([path, commit], []) = value.to_fixed_object(["path", "commit"], [])?;
        Ok(Self {
            path: path.try_to()?,
            commit: commit.try_to()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChunkEntry {
    pub path: PathBuf,
    pub line: usize,
    pub embedding: Embedding,
}

impl nojson::DisplayJson for ChunkEntry {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("path", &self.path)?;
            f.member("line", self.line)?;
            f.member("embedding", &self.embedding)
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for ChunkEntry {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([path, line, embedding], []) =
            value.to_fixed_object(["path", "line", "embedding"], [])?;
        Ok(Self {
            path: path.try_to()?,
            line: line.try_to()?,
            embedding: embedding.try_to()?,
        })
    }
}
