use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    chunker::Chunker,
    embedder::Embedder,
    git::GitRepository,
    index_file::{ChunkEntry, IndexFile, IndexFileEntry, RepositoryEntry},
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
    let dry_run = noargs::flag("dry-run").take(&mut args).is_present();
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let repo = GitRepository::new(&repo_path).or_fail()?;
    eprintln!("Target repository: {}", repo_path.display());

    let index_file = IndexFile::load(&index_file_path).or_fail()?;

    // Find the repository entry in the index
    let mut repo_entry = None;
    for r in index_file.repositories() {
        let r = r.or_fail()?;
        if r.path == repo.root_dir {
            repo_entry = Some(r);
            break;
        }
    }

    let repo_entry =
        repo_entry.or_fail_with(|()| "Repository has not been added to index".to_owned())?;

    let current_commit = repo.commit_hash().or_fail()?;
    eprintln!("Current commit hash: {}", current_commit);
    eprintln!("Indexed commit hash: {}", repo_entry.commit);

    if current_commit == repo_entry.commit {
        eprintln!("Repository is already up to date");
        return Ok(());
    }

    if dry_run {
        eprintln!(
            "Would sync repository from {} to {}",
            repo_entry.commit, current_commit
        );
        return Ok(());
    }

    // Create a temporary index file
    let temp_index_file =
        IndexFile::create_new(index_file.path.with_extension("temp")).or_fail()?;

    // Copy all entries except the ones for this repository
    let mut found_repo = false;
    for entry in index_file.entries() {
        let entry = entry.or_fail()?;
        match entry {
            IndexFileEntry::Repository(r) => {
                if r.path == repo.root_dir {
                    found_repo = true;
                    // Update the repository entry with new commit hash
                    let updated_repo = RepositoryEntry {
                        commit: current_commit.clone(),
                        ..r
                    };
                    temp_index_file.append_repository(&updated_repo).or_fail()?;
                } else {
                    temp_index_file.append_repository(&r).or_fail()?;
                }
            }
            IndexFileEntry::Chunk(c) => {
                // Skip chunks for the repository we're syncing
                if found_repo
                    && c.path.starts_with(
                        &repo
                            .root_dir
                            .strip_prefix(&repo.root_dir)
                            .unwrap_or(&repo.root_dir),
                    )
                {
                    continue;
                }
                temp_index_file.append_chunk(&c).or_fail()?;
            }
        }
    }

    // Re-index all files for this repository
    let chunker = Chunker::new(repo_entry.chunk_window_size, repo_entry.chunk_step_size);
    let embedder = Embedder::new(api_key, model);

    eprintln!("Re-indexing files...");
    for file_path in repo.files().or_fail()? {
        let abs_file_path = repo.root_dir.join(&file_path);
        let excluded = repo_entry
            .exclude_files
            .iter()
            .any(|p| p.matches(&abs_file_path))
            || repo_entry
                .include_files
                .iter()
                .all(|p| !p.matches(&abs_file_path));

        if excluded {
            eprintln!("Excluded file: {}", file_path.display());
            continue;
        }

        eprintln!("Processing file: {}", file_path.display());

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
            temp_index_file
                .append_chunk(&ChunkEntry {
                    path: file_path.clone(),
                    line: chunk.line,
                    embedding,
                })
                .or_fail()?;
        }
    }

    // Replace the original index file with the updated one
    std::fs::rename(temp_index_file.path, index_file.path).or_fail()?;

    eprintln!("=> Synced repository to commit {}", current_commit);
    Ok(())
}
