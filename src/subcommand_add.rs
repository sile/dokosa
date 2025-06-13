use std::path::PathBuf;

use orfail::OrFail;

use crate::{
    chunker::{Chunk, Chunker},
    embedder::Embedder,
    git::GitRepository,
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
    // --dry-run, --include-files, --exclude-files
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

fn glob_matches(s: &str, pattern: &str) -> bool {
    let s_chars: Vec<char> = s.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    fn matches_helper(s: &[char], pattern: &[char]) -> bool {
        match (s.is_empty(), pattern.is_empty()) {
            // Both empty - match
            (true, true) => true,
            // String empty but pattern has non-* characters - no match
            (true, false) => pattern.iter().all(|&c| c == '*'),
            // Pattern empty but string has characters - no match
            (false, true) => false,
            // Both have characters
            (false, false) => {
                match pattern[0] {
                    '*' => {
                        // Try matching * with empty string (skip the *)
                        matches_helper(s, &pattern[1..]) ||
                        // Try matching * with one or more characters
                        matches_helper(&s[1..], pattern)
                    }
                    c => {
                        // Regular character must match exactly
                        s[0] == c && matches_helper(&s[1..], &pattern[1..])
                    }
                }
            }
        }
    }

    matches_helper(&s_chars, &pattern_chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matches() {
        // Exact matches
        assert!(glob_matches("hello", "hello"));
        assert!(!glob_matches("hello", "world"));

        // Multi-character wildcard
        assert!(glob_matches("hello", "*"));
        assert!(glob_matches("hello", "h*"));
        assert!(glob_matches("hello", "*o"));
        assert!(glob_matches("hello", "h*o"));
        assert!(glob_matches("hello", "*ell*"));
        assert!(glob_matches("hello", "he*lo"));
        assert!(!glob_matches("hello", "*x*"));
        assert!(!glob_matches("hello", "h*x"));

        // Multiple stars
        assert!(glob_matches("hello", "h**o"));
        assert!(glob_matches("hello", "*h*e*l*l*o*"));
        assert!(glob_matches("abc", "***"));

        // Edge cases
        assert!(glob_matches("", ""));
        assert!(glob_matches("", "*"));
        assert!(glob_matches("", "***"));
        assert!(!glob_matches("a", ""));
        assert!(!glob_matches("hello", "hi*"));
    }
}
