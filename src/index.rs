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

    pub fn add(&mut self, repo: IndexedRepository) {
        self.repositories.insert(repo.path.clone(), repo);
    }

    pub fn save(&self) -> orfail::Result<()> {
        let json = nojson::json(|f| {
            f.object(|f| f.members(self.repositories.iter().map(|x| (x.0.display(), x.1))))
        })
        .to_string();
        std::fs::write(&self.path, json).or_fail()?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &Embedding,
        count: usize,
        similarity_threshold: f64,
    ) -> orfail::Result<Vec<Chunk<ChunkInfo>>> {
        let mut candidates = Vec::new();

        // Collect all chunks with their similarity scores
        for (root_dir, repo) in self.repositories.iter() {
            for file in &repo.files {
                for chunk in &file.chunks {
                    let similarity = self.cosine_similarity(query, &chunk.data);
                    if similarity < similarity_threshold {
                        continue;
                    }
                    candidates.push(Chunk {
                        line: chunk.line,
                        data: ChunkInfo {
                            file_path: file.path.clone(), // TODO: remove clone
                            similarity,
                            relative_file_path: root_dir
                                .parent()
                                .or_fail()?
                                .strip_prefix(&file.path)
                                .or_fail()?
                                .to_path_buf(),
                        },
                    });
                }
            }
        }

        // Sort by similarity (highest first) and take top count
        candidates.sort_by(|a, b| {
            b.data
                .similarity
                .partial_cmp(&a.data.similarity)
                .expect("TODO")
        });
        Ok(candidates.into_iter().take(count).collect())
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

#[derive(Debug)]
pub struct ChunkInfo {
    pub file_path: PathBuf,
    pub relative_file_path: PathBuf,
    pub similarity: f64,
}

#[derive(Debug)]
pub struct IndexedRepository {
    pub path: PathBuf,
    pub commit: String,
    pub files: Vec<ChunkedFile>,
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
