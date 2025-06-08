use std::{num::NonZeroUsize, path::PathBuf, time::Duration};

#[derive(Debug)]
pub struct Chunker {
    pub window_size: NonZeroUsize,
    pub step_size: NonZeroUsize,
}

impl Chunker {
    pub fn new() -> Self {
        Self {
            window_size: NonZeroUsize::MIN.saturating_add(99),
            step_size: NonZeroUsize::MIN.saturating_add(49),
        }
    }
}

#[derive(Debug)]
pub struct ChunkedFile<T> {
    pub path: PathBuf,
    pub time: Duration,
    pub labels: Vec<String>,
    pub chunks: Vec<Chunk<T>>,
}

#[derive(Debug)]
pub struct Chunk<T> {
    pub line: NonZeroUsize,
    pub data: T,
}
