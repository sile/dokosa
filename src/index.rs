use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use orfail::OrFail;

use crate::{chunker::Chunk, embedder::Embedding};

#[derive(Debug)]
pub struct IndexFile {
    path: PathBuf,
    repositories: BTreeMap<PathBuf, IndexedRepository>,
}

impl IndexFile {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let json = std::fs::read_to_string(&path).or_fail()?;
            Ok(Self {
                path,
                repositories: json.parse().map(|nojson::Json(v)| v).or_fail()?,
            })
        } else {
            Ok(Self {
                path,
                repositories: BTreeMap::new(),
            })
        }
    }

    pub fn save(&self) -> orfail::Result<()> {
        let json = nojson::json(|f| {
            f.object(|f| f.members(self.repositories.iter().map(|x| (x.0.display(), x.1))))
        })
        .to_string();
        std::fs::write(&self.path, json).or_fail()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct IndexedRepository {
    pub path: PathBuf,
    pub commit: String,
    pub files: ChunkedFile,
}

impl nojson::DisplayJson for IndexedRepository {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("path", &self.path)?;
            f.member("commit", &self.commit)?;
            f.member("files", &self.files)
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for IndexedRepository {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([path, commit, files], []) = value.to_fixed_object(["path", "commit", "files"], [])?;
        Ok(IndexedRepository {
            path: path.try_to()?,
            commit: commit.try_to()?,
            files: files.try_to()?,
        })
    }
}

#[derive(Debug)]
pub struct ChunkedFile {
    pub path: PathBuf,
    pub chunks: Vec<Chunk<Embedding>>,
}

impl nojson::DisplayJson for ChunkedFile {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("path", &self.path)?;
            f.member("chunks", &self.chunks)
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for ChunkedFile {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([path, chunks], []) = value.to_fixed_object(["path", "chunks"], [])?;
        Ok(ChunkedFile {
            path: path.try_to()?,
            chunks: chunks.try_to()?,
        })
    }
}
