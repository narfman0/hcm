use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub repo_root: PathBuf,
}

pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(start)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

pub fn repo_slug(repo_root: &Path) -> String {
    sanitize(
        repo_root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "repo".to_string())
            .as_str(),
    )
}

pub fn create(workspace: &Path, repo_root: &Path, session_name: &str) -> Result<Worktree> {
    let session_slug = sanitize(session_name);
    let branch = format!("hcm/{}", session_slug);
    let dir_name = format!("{}-{}", repo_slug(repo_root), session_slug);
    let path = workspace.join("worktrees").join(dir_name);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("failed to create worktree parent directory")?;
    }

    let new_branch = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["worktree", "add", "-b", &branch])
        .arg(&path)
        .output()
        .context("failed to invoke git worktree add")?;

    if !new_branch.status.success() {
        let stderr = String::from_utf8_lossy(&new_branch.stderr);
        if stderr.contains("already exists") || stderr.contains("already used") {
            let reuse = Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .args(["worktree", "add"])
                .arg(&path)
                .arg(&branch)
                .output()
                .context("failed to invoke git worktree add (reuse branch)")?;
            if !reuse.status.success() {
                anyhow::bail!(
                    "git worktree add failed: {}",
                    String::from_utf8_lossy(&reuse.stderr)
                );
            }
        } else {
            anyhow::bail!("git worktree add failed: {}", stderr);
        }
    }

    Ok(Worktree {
        path,
        branch,
        repo_root: repo_root.to_path_buf(),
    })
}

pub fn remove(wt: &Worktree) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(&wt.repo_root)
        .args(["worktree", "remove", "--force"])
        .arg(&wt.path)
        .output()
        .context("failed to invoke git worktree remove")?;

    if !output.status.success() {
        anyhow::bail!(
            "git worktree remove failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}
