use std::{num::NonZeroUsize, path::PathBuf};

use orfail::OrFail;

use crate::{
    chunker::{Chunk, Chunker},
    embedder::Embedder,
    git::GitRepository,
    glob::GlobPathPattern,
    index_file::IndexFile,
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
    let mut include_files = Vec::new();
    while let Some(a) = noargs::opt("include-files")
        .short('I')
        .ty("PATTERN")
        .take(&mut args)
        .present()
    {
        include_files.push(GlobPathPattern::new(a.value()));
    }
    if include_files.is_empty() {
        include_files.push(GlobPathPattern::new("*"));
    }
    let mut exclude_files = Vec::new();
    while let Some(a) = noargs::opt("exclude-files")
        .short('E')
        .ty("PATTERN")
        .take(&mut args)
        .present()
    {
        exclude_files.push(GlobPathPattern::new(a.value()));
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

    for file_path in repo.files().or_fail()? {
        let abs_file_path = repo.root_dir.join(&file_path);
        let excluded = exclude_files.iter().any(|p| p.matches(&abs_file_path))
            || include_files.iter().all(|p| !p.matches(&abs_file_path));
        if excluded {
            eprintln!("Excluded file: {}", file_path.display());
            continue;
        }

        eprintln!("Included file: {}", file_path.display());
    }

    if dry_run {
        return Ok(());
    }

    // let chunker = Chunker::new();
    // let embedder = Embedder::new(api_key, model);

    // let mut files = Vec::new();
    // for file_path in repo.files().or_fail()? {
    //     println!("# FILE: {}", file_path.display());

    //     let Ok(content) = std::fs::read_to_string(&file_path)
    //         .inspect_err(|e| eprintln!("Failed to read file {}: {}", file_path.display(), e))
    //     else {
    //         continue;
    //     };

    //     let chunks = chunker.apply(&content);
    //     let inputs = chunks.iter().map(|c| c.data.clone()).collect::<Vec<_>>(); // TODO: remove clone
    //     let Ok(embeddings) = embedder
    //         .embed(&inputs)
    //         .or_fail()
    //         .inspect_err(|e| eprintln!("Failed to embed: {e}"))
    //     else {
    //         continue;
    //     };
    //     let chunks = chunks
    //         .iter()
    //         .zip(embeddings)
    //         .map(|(c, e)| Chunk {
    //             line: c.line,
    //             data: e,
    //         })
    //         .collect::<Vec<_>>();
    //     files.push(ChunkedFile {
    //         path: file_path, // TODO: relative path
    //         chunks,
    //     });
    // }

    // indexer.add(IndexedRepository {
    //     path: repo.root_dir.clone(),
    //     commit: repo.commit_hash().or_fail()?,
    //     files,
    // });

    // eprintln!("# SAVE");
    // indexer.save().or_fail()?;

    Ok(())
}
