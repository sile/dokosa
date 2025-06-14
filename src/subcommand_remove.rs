use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    git::GitRepository,
    index_file::{IndexFile, IndexFileEntry},
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
    let dry_run = noargs::flag("dry-run").take(&mut args).is_present();
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let repo = GitRepository::new(&repo_path).or_fail()?;
    eprintln!("Target repository: {}", repo_path.display());

    let index_file = IndexFile::load(&index_file_path).or_fail()?;
    index_file
        .repositories()
        .any(|r| r.is_ok_and(|r| r.path == repo.root_dir))
        .or_fail_with(|()| "Repository has not been added".to_owned())?;

    if dry_run {
        return Ok(());
    }

    let temp_index_file =
        IndexFile::create_new(index_file.path.with_extension("temp")).or_fail()?;
    let mut removing = false;
    for entry in index_file.entries() {
        let entry = entry.or_fail()?;
        match entry {
            IndexFileEntry::Repository(x) => {
                if x.path == repo.root_dir {
                    removing = true;
                } else {
                    removing = false;
                    index_file.append_repository(&x).or_fail()?;
                }
            }
            IndexFileEntry::Chunk(x) => {
                if removing {
                    continue;
                }
                index_file.append_chunk(&x).or_fail()?;
            }
        }
    }
    std::fs::rename(temp_index_file.path, index_file.path).or_fail()?;

    eprintln!("=> Removed");
    Ok(())
}
