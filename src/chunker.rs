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

    pub fn apply(&self, input: &str) -> Vec<Chunk<String>> {
        let mut chunks = Vec::new();
        for (i, lines) in input
            .lines()
            .collect::<Vec<_>>()
            .windows(self.window_size.get())
            .enumerate()
        {
            if i % self.step_size.get() != 0 {
                continue;
            }

            chunks.push(Chunk {
                line: i,
                data: lines.join("\n"),
            });
        }
        chunks
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
    pub line: usize,
    pub data: T,
}
