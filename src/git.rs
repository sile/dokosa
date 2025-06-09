use std::{
    path::{Path, PathBuf},
    process::Command,
};

use orfail::OrFail;

#[derive(Debug)]
pub struct GitRepository {
    root_dir: PathBuf,
}

impl GitRepository {
    /// Create a new GitRepository instance for the given path
    /// Returns an error if the path is not a valid Git repository
    pub fn new<P: AsRef<Path>>(repository_path: P) -> orfail::Result<Self> {
        let path = repository_path.as_ref();

        // Verify it's a valid Git repository and get the root directory
        let output = Command::new("git")
            .args(&[
                "-C",
                path.to_str().unwrap_or(""),
                "rev-parse",
                "--show-toplevel",
            ])
            .output()
            .or_fail_with(|e| format!("Failed to execute git rev-parse --show-toplevel: {e}"))?;

        output.status.success().or_fail_with(|()| {
            format!(
                "Not a valid Git repository: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

        let root_dir = String::from_utf8(output.stdout)
            .or_fail()?
            .trim()
            .to_string();

        Ok(GitRepository {
            root_dir: PathBuf::from(root_dir),
        })
    }

    /// Get the current commit hash (HEAD)
    pub fn commit_hash(&self) -> orfail::Result<String> {
        let output = Command::new("git")
            .args(&[
                "-C",
                self.root_dir.to_str().unwrap_or(""),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .or_fail_with(|e| format!("Failed to execute git rev-parse HEAD: {e}"))?;

        output.status.success().or_fail_with(|()| {
            format!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

        let commit_hash = String::from_utf8(output.stdout)
            .or_fail()?
            .trim()
            .to_string();

        Ok(commit_hash)
    }

    /// Get all files tracked by Git in this repository
    pub fn files(&self) -> orfail::Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .args(&["-C", self.root_dir.to_str().unwrap_or(""), "ls-files"])
            .output()
            .or_fail_with(|e| format!("Failed to execute git ls-files: {e}"))?;

        output.status.success().or_fail_with(|()| {
            format!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

        let files_str = String::from_utf8(output.stdout).or_fail()?;
        let files: Vec<PathBuf> = files_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.root_dir.join(line.trim()))
            .collect();

        Ok(files)
    }
}
