use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    chunker::Chunker,
    embedder::Embedder,
    git::GitRepository,
    index_file::{ChunkEntry, IndexFile, IndexFileEntry, RepositoryEntry},
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
    let mut removing = false;
    let mut updated_files = Vec::new();
    let mut removed_files = Vec::new();
    for entry in index_file.entries() {
        let entry = entry.or_fail()?;
        match entry {
            IndexFileEntry::Repository(repo) => {
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
                if repo.commit != new_commit {
                    eprintln!("  => New commit: {}", new_commit);
                }

                (updated_files, removed_files) = git.diff_files(&repo.commit).or_fail()?;
                for updated_file in &updated_files {
                    eprintln!("  => Updated file: {}", updated_file.display());
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
            }
        }
    }

    eprintln!("=> Synced");
    Ok(())
}
