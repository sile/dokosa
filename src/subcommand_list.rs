use std::path::PathBuf;

use orfail::OrFail;

use crate::index_file::{IndexFile, IndexFileEntry};

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let index_file_path: PathBuf = noargs::opt("index-file")
        .short('i')
        .ty("PATH")
        .env("DOKOSA_INDEX_FILE")
        .example("/path/to/.dokosa")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let index_file = IndexFile::load(&index_file_path).or_fail()?;

    let mut repo_count = 0;
    let mut chunk_count = 0;

    for entry in index_file.entries() {
        let entry = entry.or_fail()?;
        match entry {
            IndexFileEntry::Repository(repo) => {
                repo_count += 1;
                println!("Repository: {}", repo.path.display());
                println!("  Commit: {}", repo.commit);
            }
            IndexFileEntry::Chunk(chunk) => {
                chunk_count += 1;
                println!("  Chunk: {} (line {})", chunk.path.display(), chunk.line);
            }
        }
    }

    println!("\nSummary:");
    println!("  Repositories: {}", repo_count);
    println!("  Chunks: {}", chunk_count);

    Ok(())
}
