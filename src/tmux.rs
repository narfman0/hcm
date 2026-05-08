use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::worktree::Worktree;

pub type SessionId = String;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub cmd: String,
    pub running: bool,
    pub created_at: SystemTime,
    pub cwd: PathBuf,
    pub worktree: Option<Worktree>,
}

pub struct SpawnOptions<'a> {
    pub name: &'a str,
    pub cmd: &'a str,
    pub cwd: &'a Path,
}

pub struct Tmux;

impl Tmux {
    pub fn new() -> Self {
        Self
    }

    /// Bind Ctrl-\ (no prefix) to detach-client so users can pop out of an
    /// attached session back to the hcm dashboard with a single chord.
    /// Requires a running tmux server, so this is called lazily after
    /// session-creating commands rather than at construction time.
    fn ensure_detach_binding(&self) {
        let _ = Command::new("tmux")
            .args(["bind-key", "-n", "C-\\", "detach-client"])
            .status();
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        let output = Command::new("tmux")
            .args([
                "list-sessions",
                "-F",
                "#{session_id}|#{session_name}|#{session_created}|#{session_path}",
            ])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return vec![],
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() < 4 {
                    return None;
                }
                let id = parts[0].to_string();
                let name = parts[1].to_string();
                let created_secs: u64 = parts[2].parse().ok()?;
                let created_at = UNIX_EPOCH + Duration::from_secs(created_secs);
                let cwd = PathBuf::from(parts[3]);

                Some(Session {
                    id,
                    name,
                    cmd: String::new(),
                    running: true,
                    created_at,
                    cwd,
                    worktree: None,
                })
            })
            .collect()
    }

    pub fn spawn_session(&self, opts: SpawnOptions) -> Result<SessionId> {
        let cwd_str = opts.cwd.to_string_lossy().into_owned();
        let mut args: Vec<String> = vec![
            "new-session".into(),
            "-d".into(),
            "-s".into(),
            opts.name.into(),
            "-c".into(),
            cwd_str,
        ];
        for tok in opts.cmd.split_whitespace() {
            args.push(tok.to_string());
        }

        let status = Command::new("tmux")
            .args(args.iter().map(|s| s.as_str()))
            .status()
            .context("failed to run tmux new-session")?;

        if status.success() {
            self.ensure_detach_binding();
            Ok(opts.name.to_string())
        } else {
            Err(anyhow::anyhow!("tmux new-session failed"))
        }
    }

    pub fn rename_session(&self, id: &SessionId, new_name: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["rename-session", "-t", id.as_str(), new_name])
            .status()
            .context("failed to run tmux rename-session")?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("tmux rename-session failed"))
        }
    }

    pub fn attach_session(&self, id: &SessionId) -> Result<()> {
        self.ensure_detach_binding();

        let in_tmux = std::env::var("TMUX").is_ok();
        let (subcmd, flag) = if in_tmux {
            ("switch-client", "-t")
        } else {
            ("attach-session", "-t")
        };

        let status = Command::new("tmux")
            .args([subcmd, flag, id.as_str()])
            .status()
            .context("failed to run tmux attach/switch")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("tmux {} failed", subcmd))
        }
    }

    pub fn kill_session(&self, id: &SessionId) -> Result<()> {
        let status = Command::new("tmux")
            .args(["kill-session", "-t", id.as_str()])
            .status()
            .context("failed to run tmux kill-session")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("tmux kill-session failed"))
        }
    }

}
