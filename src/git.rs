use std::{
    path::{Path, PathBuf},
    process::Command,
};

use orfail::OrFail;

pub fn is_available<P: AsRef<Path>>(repository_path: P) -> bool {
    // Check if the given path is a Git repository by trying to get the top-level directory
    let output = Command::new("git")
        .args(&[
            "-C",
            repository_path.as_ref().to_str().unwrap_or(""),
            "rev-parse",
            "--show-toplevel",
        ])
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn get_commit_hash<P: AsRef<Path>>(repository_path: P) -> orfail::Result<String> {
    let output = Command::new("git")
        .args(&[
            "-C",
            repository_path.as_ref().to_str().unwrap_or(""),
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

pub fn get_files<P: AsRef<Path>>(repository_path: P) -> orfail::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(&[
            "-C",
            repository_path.as_ref().to_str().unwrap_or(""),
            "ls-files",
        ])
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
        .map(|line| PathBuf::from(line.trim()))
        .collect();

    Ok(files)
}
