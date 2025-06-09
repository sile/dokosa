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
