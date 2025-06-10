use std::{io::Read, path::PathBuf};

use orfail::OrFail;

use crate::{embedder::Embedder, index::IndexFile};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let index_path: PathBuf = noargs::opt("index-file")
        .short('i')
        .env("DOKOSA_INDEX_PATH")
        .default(".dokosa")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let count: usize = noargs::opt("count")
        .short('c')
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
        .example("YOUR_API_KEY")
        .env("OPENAI_API_KEY")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let model: String = noargs::opt("model")
        .ty("STRING")
        .default("text-embedding-3-small")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let indexer = IndexFile::load_or_create(&index_path).or_fail()?;
    let embedder = Embedder::new(api_key, model);

    let mut query = String::new();
    std::io::stdin().read_to_string(&mut query).or_fail()?;

    let embedding = embedder.embed(&[query]).or_fail()?.remove(0);
    let chunks = indexer
        .search(&embedding, count, similarity_threshold)
        .or_fail()?;

    let mut contents = Vec::new();
    for chunk in chunks {
        let content = std::fs::read_to_string(&chunk.data.file_path).or_fail()?;
        contents.push(SimilarContent {
            similarity: chunk.data.similarity,
            path: chunk.data.relative_file_path,
            line: chunk.line,
            content: content
                .lines()
                .skip(chunk.line)
                .take(100) // TODO: remove magic value
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }

    println!("{}", nojson::Json(&contents));
    Ok(())
}

#[derive(Debug)]
struct SimilarContent {
    similarity: f64,
    path: PathBuf,
    line: usize,
    content: String,
}

impl nojson::DisplayJson for SimilarContent {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("similarity", self.similarity)?;
            f.member("path", &self.path)?;
            f.member("line", self.line)?;
            f.member("content", &self.content)
        })
    }
}
