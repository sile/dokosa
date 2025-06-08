use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Chunker {
    pub window_size: NonZeroUsize,
    pub step_size: NonZeroUsize,
}

#[derive(Debug)]
pub struct Chunk {}
