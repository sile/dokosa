use std::{io::Read, path::PathBuf};

use orfail::OrFail;

use crate::{
    embedder::Embedder,
    glob::{GlobPathFilter, GlobPathPattern},
    index_file::IndexFile,
};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let index_file_path: PathBuf = noargs::opt("index-file")
        .short('i')
        .ty("PATH")
        .doc("Path to the index file to search within")
        .env("DOKOSA_INDEX_FILE")
        .example("/path/to/.dokosa")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let count: usize = noargs::opt("count")
        .short('c')
        .ty("NUMBER")
        .doc("Maximum number of search results to return")
        .default("10")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let similarity_threshold: f64 = noargs::opt("similarity-threshold")
        .short('t')
        .ty("FLOAT")
        .default("0.3")
        .doc("Minimum similarity score (0.0 to 1.0) for results to be included")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let api_key: String = noargs::opt("openai-api-key")
        .ty("STRING")
        .doc("OpenAI API key for generating embeddings")
        .example("YOUR_API_KEY")
        .env("OPENAI_API_KEY")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let model: String = noargs::opt("embedding-model")
        .ty("STRING")
        .doc("OpenAI embedding model to use for text vectorization")
        .default("text-embedding-3-small")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let strip_text: bool = noargs::flag("strip-text")
        .doc("Exclude text content from results, returning only metadata")
        .take(&mut args)
        .is_present();
    let mut filter = GlobPathFilter::default();
    while let Some(a) = noargs::opt("include-files")
        .short('I')
        .ty("PATTERN")
        .doc("Include files matching this glob pattern (can be used multiple times)")
        .take(&mut args)
        .present()
    {
        filter.include_files.push(GlobPathPattern::new(a.value()));
    }
    while let Some(a) = noargs::opt("exclude-files")
        .short('E')
        .ty("PATTERN")
        .doc("Exclude files matching this glob pattern (can be used multiple times)")
        .take(&mut args)
        .present()
    {
        filter.exclude_files.push(GlobPathPattern::new(a.value()));
    }
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let index_file = IndexFile::load(&index_file_path).or_fail()?;
    let embedder = Embedder::new(api_key, model);

    let mut query = String::new();
    std::io::stdin().read_to_string(&mut query).or_fail()?;

    let embedding = embedder.embed(&[query]).or_fail()?.remove(0);
    let matched_chunks = index_file
        .search(&embedding, count, similarity_threshold, &filter)
        .or_fail()?;

    let mut chunks = Vec::new();
    for chunk in matched_chunks {
        chunks.push(SimilarChunk {
            similarity: chunk.similarity,
            path: chunk.repository_file_path().or_fail()?,
            line: chunk.line,
            text: if strip_text {
                "".to_owned()
            } else {
                chunk.chunk_text().or_fail()?
            },
        });
    }

    println!("{}", nojson::Json(&chunks));
    Ok(())
}

#[derive(Debug)]
struct SimilarChunk {
    similarity: f64,
    path: PathBuf,
    line: usize,
    text: String,
}

impl nojson::DisplayJson for SimilarChunk {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("similarity", self.similarity)?;
            f.member("path", &self.path)?;
            f.member("line", self.line)?;
            f.member("text", &self.text)
        })
    }
}
