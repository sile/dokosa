use std::{
    io::{BufRead, BufWriter, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use orfail::OrFail;

use crate::{embedder::Embedding, glob::GlobPathPattern};

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
            std::fs::File::create_new(&path).or_fail()?;
            Ok((true, Self { path }))
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let path = path.as_ref().to_path_buf();
        path.exists().or_fail()?;
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

    pub fn repositories(&self) -> Repositories {
        Repositories {
            path: self.path.clone(),
            lines: None,
        }
    }
}

#[derive(Debug)]
pub struct Repositories {
    path: PathBuf,
    lines: Option<std::io::Lines<std::io::BufReader<std::fs::File>>>,
}

impl Repositories {
    fn next_entry(&mut self) -> orfail::Result<Option<RepositoryEntry>> {
        let Some(lines) = &mut self.lines else {
            let file = std::fs::File::open(&self.path).or_fail()?;
            let reader = std::io::BufReader::new(file);
            self.lines = Some(reader.lines());
            return self.next_entry();
        };

        loop {
            let Some(line) = lines.next().transpose().or_fail()? else {
                return Ok(None);
            };
            if let IndexFileEntry::Repository(x) =
                line.parse().map(|nojson::Json(x)| x).or_fail()?
            {
                return Ok(Some(x));
            }
        }
    }
}

impl Iterator for Repositories {
    type Item = orfail::Result<RepositoryEntry>;

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
