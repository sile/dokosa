use std::{
    io::{BufRead, BufWriter, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use orfail::OrFail;

use crate::{
    embedder::Embedding,
    glob::{GlobPathFilter, GlobPathPattern},
};

#[derive(Debug)]
pub struct IndexFile {
    pub path: PathBuf,
}

impl IndexFile {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> orfail::Result<(bool, Self)> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            Self::load(path).or_fail().map(|this| (false, this))
        } else {
            Self::create_new(path).or_fail().map(|this| (true, this))
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        path.exists().or_fail()?;
        Ok(Self { path })
    }

    pub fn create_new<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::File::create_new(&path).or_fail()?;
        Ok(Self { path })
    }

    pub fn append_repository(&self, repo: &RepositoryEntry) -> orfail::Result<()> {
        self.append(repo).or_fail()
    }

    pub fn append_chunk(&self, chunk: &ChunkEntry) -> orfail::Result<()> {
        self.append(chunk).or_fail()
    }

    fn append<T: nojson::DisplayJson>(&self, entry: &T) -> orfail::Result<()> {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .or_fail()?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", nojson::Json(entry)).or_fail()?;
        writer.flush().or_fail()?;
        Ok(())
    }

    pub fn entries(&self) -> impl Iterator<Item = orfail::Result<IndexFileEntry>> {
        Entries {
            path: self.path.clone(),
            lines: None,
        }
    }

    pub fn repositories(&self) -> impl Iterator<Item = orfail::Result<RepositoryEntry>> {
        self.entries().filter_map(|x| match x {
            Err(e) => Some(Err(e)),
            Ok(IndexFileEntry::Repository(x)) => Some(Ok(x)),
            _ => None,
        })
    }

    pub fn search(
        &self,
        query: &Embedding,
        count: usize,
        similarity_threshold: f64,
        filter: &GlobPathFilter,
    ) -> orfail::Result<Vec<MatchedChunk>> {
        let mut candidates = Vec::new();
        let mut lowest_similarity = similarity_threshold.next_down();
        let mut repository = None;

        // Collect all chunk entries with their similarity scores
        for entry_result in self.entries() {
            let entry = entry_result.or_fail()?;
            match entry {
                IndexFileEntry::Repository(repo) => {
                    repository = Some(repo);
                }
                IndexFileEntry::Chunk(chunk) => {
                    let repository = repository.as_ref().or_fail()?;
                    if !filter.matches(repository.path.join(&chunk.path)) {
                        continue;
                    }
                    let similarity = self.cosine_similarity(query, &chunk.embedding);
                    if similarity > lowest_similarity {
                        candidates.push(MatchedChunk {
                            repository_path: repository.path.clone(),
                            chunk_window_size: repository.chunk_window_size,
                            file_path: chunk.path,
                            line: chunk.line,
                            similarity,
                        });

                        candidates.sort_by(|a, b| {
                            b.similarity
                                .partial_cmp(&a.similarity)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        if candidates.len() > count {
                            let lowest = candidates.pop().expect("infallible");
                            lowest_similarity = lowest.similarity;
                        }
                    }
                }
            }
        }

        Ok(candidates)
    }

    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f64 {
        let a_vec = &a.0;
        let b_vec = &b.0;

        if a_vec.len() != b_vec.len() {
            return 0.0;
        }

        let dot_product: f64 = a_vec.iter().zip(b_vec.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b_vec.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[derive(Debug, Clone)]
pub struct MatchedChunk {
    pub repository_path: PathBuf,
    pub chunk_window_size: NonZeroUsize,
    pub file_path: PathBuf,
    pub line: usize,
    pub similarity: f64,
}

impl MatchedChunk {
    pub fn relative_file_path(&self, current_dir: &Path) -> PathBuf {
        let full_path = self.repository_path.join(&self.file_path);

        if let Ok(relative_path) = full_path.strip_prefix(current_dir) {
            // The file is within the current directory tree, return the direct relative path
            return relative_path.to_path_buf();
        }

        // The file is outside the current directory tree, compute path via common ancestor
        let common_len = current_dir
            .components()
            .zip(full_path.components())
            .take_while(|(a, b)| a == b)
            .count();

        if common_len == 0 {
            // No common ancestor
            return full_path;
        }

        current_dir
            .components()
            .skip(common_len)
            .map(|_| std::path::Component::ParentDir)
            .chain(full_path.components().skip(common_len))
            .collect()
    }

    pub fn chunk_text(&self) -> orfail::Result<String> {
        let full_path = self.repository_path.join(&self.file_path);
        let text = std::fs::read_to_string(&full_path)
            .or_fail_with(|e| format!("{e}: {}", full_path.display()))?;
        Ok(text
            .lines()
            .skip(self.line)
            .take(self.chunk_window_size.get())
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

#[derive(Debug)]
struct Entries {
    path: PathBuf,
    lines: Option<std::io::Lines<std::io::BufReader<std::fs::File>>>,
}

impl Entries {
    fn next_entry(&mut self) -> orfail::Result<Option<IndexFileEntry>> {
        let Some(lines) = &mut self.lines else {
            let file = std::fs::File::open(&self.path).or_fail()?;
            let reader = std::io::BufReader::new(file);
            self.lines = Some(reader.lines());
            return self.next_entry();
        };

        let Some(line) = lines.next().transpose().or_fail()? else {
            return Ok(None);
        };

        let entry: IndexFileEntry = line.parse().map(|nojson::Json(x)| x).or_fail()?;
        Ok(Some(entry))
    }
}

impl Iterator for Entries {
    type Item = orfail::Result<IndexFileEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entry().or_fail().transpose()
    }
}

#[derive(Debug, Clone)]
pub enum IndexFileEntry {
    Repository(RepositoryEntry),
    Chunk(ChunkEntry),
}

impl nojson::DisplayJson for IndexFileEntry {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        match self {
            IndexFileEntry::Repository(repo) => repo.fmt(f),
            IndexFileEntry::Chunk(chunk) => chunk.fmt(f),
        }
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for IndexFileEntry {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([entry_type], []) = value.to_fixed_object(["type"], [])?;
        match entry_type.to_unquoted_string_str()?.as_ref() {
            "repository" => Ok(IndexFileEntry::Repository(value.try_to()?)),
            "chunk" => Ok(IndexFileEntry::Chunk(value.try_to()?)),
            ty => Err(nojson::JsonParseError::invalid_value(
                value,
                format!(
                    "Invalid type field: expected 'repository' or 'chunk', found '{}'",
                    ty
                ),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryEntry {
    pub path: PathBuf,
    pub commit: String,
    pub chunk_window_size: NonZeroUsize,
    pub chunk_step_size: NonZeroUsize,
    pub include_files: Vec<GlobPathPattern>,
    pub exclude_files: Vec<GlobPathPattern>,
}

impl nojson::DisplayJson for RepositoryEntry {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("type", "repository")?;
            f.member("path", &self.path)?;
            f.member("commit", &self.commit)?;
            f.member("chunk_window_size", self.chunk_window_size)?;
            f.member("chunk_step_size", self.chunk_step_size)?;
            f.member("include_files", &self.include_files)?;
            f.member("exclude_files", &self.exclude_files)
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for RepositoryEntry {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let (
            [
                path,
                commit,
                chunk_window_size,
                chunk_step_size,
                include_files,
                exclude_files,
            ],
            [],
        ) = value.to_fixed_object(
            [
                "path",
                "commit",
                "chunk_window_size",
                "chunk_step_size",
                "include_files",
                "exclude_files",
            ],
            [],
        )?;

        Ok(Self {
            path: path.try_to()?,
            commit: commit.try_to()?,
            chunk_window_size: chunk_window_size.try_to()?,
            chunk_step_size: chunk_step_size.try_to()?,
            include_files: include_files.try_to()?,
            exclude_files: exclude_files.try_to()?,
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
            f.member("type", "chunk")?;
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
