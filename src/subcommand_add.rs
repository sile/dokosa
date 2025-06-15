use std::{num::NonZeroUsize, path::PathBuf};

use orfail::OrFail;

use crate::{
    chunker::Chunker,
    embedder::Embedder,
    git::GitRepository,
    glob::{GlobPathFilter, GlobPathPattern},
    index_file::{ChunkEntry, IndexFile, RepositoryEntry},
};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let repo_path: PathBuf = noargs::arg("GIT_REPOSITORY_PATH")
        .example("/path/to/git/repository/")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let index_file_path: PathBuf = noargs::opt("index-file")
        .short('i')
        .ty("PATH")
        .env("DOKOSA_INDEX_FILE")
        .example("/path/to/.dokosa")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let api_key: String = noargs::opt("openai-api-key")
        .ty("STRING")
        .example("YOUR_API_KEY")
        .env("OPENAI_API_KEY")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let model: String = noargs::opt("embedding-model")
        .ty("STRING")
        .default("text-embedding-3-small")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let chunk_window_size: NonZeroUsize = noargs::opt("chunk-window-size")
        .short('w')
        .ty("LINE_COUNT")
        .default("100")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let chunk_step_size: NonZeroUsize = noargs::opt("chunk-step-size")
        .short('s')
        .ty("LINE_COUNT")
        .default("50")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let dry_run = noargs::flag("dry-run").take(&mut args).is_present();

    let mut filter = GlobPathFilter::default();
    while let Some(a) = noargs::opt("include-files")
        .short('I')
        .ty("PATTERN")
        .take(&mut args)
        .present()
    {
        filter.include_files.push(GlobPathPattern::new(a.value()));
    }
    while let Some(a) = noargs::opt("exclude-files")
        .short('E')
        .ty("PATTERN")
        .take(&mut args)
        .present()
    {
        filter.exclude_files.push(GlobPathPattern::new(a.value()));
    }
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let repo = GitRepository::new(&repo_path).or_fail()?;
    eprintln!("Target repository: {}", repo_path.display());

    let (created, index_file) = IndexFile::load_or_create(&index_file_path).or_fail()?;
    if created {
        eprintln!("Created index file: {}", index_file_path.display());
    }

    for r in index_file.repositories() {
        (r.or_fail()?.path != repo.root_dir)
            .or_fail_with(|()| "Repository already exists".to_owned())?;
    }

    let commit = repo.commit_hash().or_fail()?;
    eprintln!("Commit hash: {}", commit);

    if !dry_run {
        index_file
            .append_repository(&RepositoryEntry {
                path: repo.root_dir.clone(),
                commit,
                chunk_window_size,
                chunk_step_size,
                include_files: filter.include_files.clone(),
                exclude_files: filter.exclude_files.clone(),
            })
            .or_fail()?;
    }

    let chunker = Chunker::new(chunk_window_size, chunk_step_size);
    let embedder = Embedder::new(api_key, model);

    for file_path in repo.files().or_fail()? {
        let abs_file_path = repo.root_dir.join(&file_path);
        if !filter.should_include(&abs_file_path) {
            eprintln!("Excluded file: {}", file_path.display());
            continue;
        }

        eprintln!("Included file: {}", file_path.display());
        if dry_run {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&abs_file_path)
            .or_fail()
            .inspect_err(|e| eprintln!("  Failed to read file: {}", e))
        else {
            continue;
        };

        let mut chunks = chunker.apply(&content);
        let inputs = chunks
            .iter_mut()
            .map(|c| std::mem::take(&mut c.data))
            .collect::<Vec<_>>();
        let Ok(embeddings) = embedder
            .embed(&inputs)
            .or_fail()
            .inspect_err(|e| eprintln!("  Failed to embed: {e}"))
        else {
            continue;
        };
        for (chunk, embedding) in chunks.iter().zip(embeddings) {
            index_file
                .append_chunk(&ChunkEntry {
                    path: file_path.clone(),
                    line: chunk.line,
                    embedding,
                })
                .or_fail()?;
        }
    }

    eprintln!("=> Added");
    Ok(())
}
