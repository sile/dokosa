use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    chunker::{Chunk, Chunker},
    embedder::Embedder,
    git::GitRepository,
};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let repository_path: PathBuf = noargs::arg("GIT_REPOSITORY_PATH")
        .example("/path/to/git/repository/")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let index_path: PathBuf = noargs::opt("index-file")
        .env("SAGURU_INDEX_FILE")
        .default(".saguru")
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

    let repo = GitRepository::new(repository_path).or_fail()?;
    let chunker = Chunker::new();
    let embedder = Embedder::new(api_key, model);

    for file_path in repo.files().or_fail()? {
        println!("# FILE: {}", file_path.display());

        let Ok(content) = std::fs::read_to_string(&file_path)
            .inspect_err(|e| eprintln!("Failed to read file {}: {}", file_path.display(), e))
        else {
            continue;
        };

        let chunks = chunker.apply(&content);
        let inputs = chunks.iter().map(|c| c.data.clone()).collect::<Vec<_>>(); // TODO: remove clone
        let embeddings = embedder.embed(&inputs).or_fail()?;
        let chunks = chunks
            .iter()
            .zip(embeddings)
            .map(|(c, e)| Chunk {
                line: c.line,
                data: e,
            })
            .collect::<Vec<_>>();
        dbg!(chunks);
    }

    Ok(())
}
