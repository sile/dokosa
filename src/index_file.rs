use std::path::PathBuf;

#[derive(Debug)]
pub struct IndexFile {
    pub path: PathBuf,
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
    pub chunks: usize,
}

#[derive(Debug, Clone)]
pub struct ChunkEntry {
    pub path: PathBuf,
    pub line: usize,
    pub embedding: Vec<f64>,
}
