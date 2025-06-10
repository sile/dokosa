use std::{io::Read, path::PathBuf};

use orfail::OrFail;

use crate::{embedder::Embedder, index::IndexFile};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let index_path: PathBuf = noargs::opt("index-file")
        .short('i')
        .env("SAGURU_INDEX_FILE")
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
    let chunks = indexer.search(&embedding, count, similarity_threshold);
    dbg!(chunks);

    Ok(())
}
