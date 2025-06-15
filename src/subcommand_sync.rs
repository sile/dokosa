use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    chunker::Chunker,
    embedder::Embedder,
    git::GitRepository,
    glob::GlobPathFilter,
    index_file::{ChunkEntry, IndexFile, IndexFileEntry},
};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
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

    let index_file = IndexFile::load(&index_file_path).or_fail()?;
    let temp_index_file = if dry_run {
        None
    } else {
        Some(IndexFile::create_new(index_file_path.with_extension(".temp")).or_fail()?)
    };

    let embedder = Embedder::new(api_key, model);
    let mut removing = false;
    let mut updated_files = Vec::new();
    let mut removed_files = Vec::new();
    for entry in index_file.entries() {
        let entry = entry.or_fail()?;
        match entry {
            IndexFileEntry::Repository(mut repo) => {
                eprintln!("Repository: {} ({})", repo.path.display(), repo.commit);
                let Ok(git) = GitRepository::new(&repo.path)
                    .or_fail()
                    .inspect_err(|e| eprintln!("  Not a valid Git repository: {e}"))
                else {
                    eprintln!("  => Removed");
                    removing = true;
                    continue;
                };
                removing = false;

                let new_commit = git.commit_hash().or_fail()?;
                if repo.commit == new_commit {
                    if let Some(temp) = &temp_index_file {
                        temp.append_repository(&repo).or_fail()?;
                    }
                    continue;
                }
                eprintln!("  => New commit: {}", new_commit);

                (updated_files, removed_files) = git.diff_files(&repo.commit).or_fail()?;

                repo.commit = new_commit;
                if let Some(temp) = &temp_index_file {
                    temp.append_repository(&repo).or_fail()?;
                }

                let filter = GlobPathFilter {
                    include_files: repo.include_files.clone(),
                    exclude_files: repo.exclude_files.clone(),
                };
                let chunker = Chunker::new(repo.chunk_window_size, repo.chunk_step_size);
                for updated_file in &updated_files {
                    if !filter.matches(updated_file) {
                        continue;
                    }
                    eprintln!("  => Updated file: {}", updated_file.display());

                    let Some(temp) = &temp_index_file else {
                        continue;
                    };

                    let abs_file_path = repo.path.join(updated_file);
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
                        temp.append_chunk(&ChunkEntry {
                            path: updated_file.clone(),
                            line: chunk.line,
                            embedding,
                        })
                        .or_fail()?;
                    }
                }
            }
            IndexFileEntry::Chunk(chunk) => {
                if removing {
                    continue;
                }
                if removed_files.contains(&chunk.path) {
                    if chunk.line == 0 {
                        eprintln!("  => Removed file: {}", chunk.path.display());
                    }
                    continue;
                }
                if updated_files.contains(&chunk.path) {
                    continue;
                }

                if let Some(temp) = &temp_index_file {
                    temp.append_chunk(&chunk).or_fail()?;
                }
            }
        }
    }

    if let Some(temp) = temp_index_file {
        std::fs::rename(temp.path, index_file.path).or_fail()?;
    }

    eprintln!("=> Synced");
    Ok(())
}
