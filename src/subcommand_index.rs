use std::path::PathBuf;

use orfail::OrFail;

use crate::git::GitRepository;

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
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let repo = GitRepository::new(repository_path).or_fail()?;

    Ok(())
}
