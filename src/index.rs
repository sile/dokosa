use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

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
            todo!()
        } else {
            Ok(Self {
                path,
                repositories: BTreeMap::new(),
            })
        }
    }
}

#[derive(Debug)]
pub struct IndexedRepository {
    pub path: PathBuf,
    pub commit: String,
    pub files: ChunkedFile,
}

#[derive(Debug)]
pub struct ChunkedFile {
    pub path: PathBuf,
    pub chunks: Vec<Chunk<Embedding>>,
}

impl nojson::DisplayJson for ChunkedFile {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("path", self.path.to_string_lossy().as_ref())?;
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
