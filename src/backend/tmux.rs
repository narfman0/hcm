use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

use super::{MultiplexerBackend, Session, SessionId, SpawnOptions};

pub struct TmuxBackend;

impl TmuxBackend {
    pub fn new() -> Self {
        Self
    }
}

impl MultiplexerBackend for TmuxBackend {
    fn list_sessions(&self) -> Vec<Session> {
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

    fn spawn_session(&self, opts: SpawnOptions) -> Result<SessionId> {
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
            Ok(opts.name.to_string())
        } else {
            Err(anyhow::anyhow!("tmux new-session failed"))
        }
    }

    fn rename_session(&self, id: &SessionId, new_name: &str) -> Result<()> {
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

    fn attach_session(&self, id: &SessionId) -> Result<()> {
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

    fn kill_session(&self, id: &SessionId) -> Result<()> {
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

    fn resize_session(&self, id: &SessionId, rows: u16, cols: u16) -> Result<()> {
        let cols_s = cols.to_string();
        let rows_s = rows.to_string();
        let status = Command::new("tmux")
            .args([
                "resize-window",
                "-t",
                id.as_str(),
                "-x",
                &cols_s,
                "-y",
                &rows_s,
            ])
            .status()
            .context("failed to run tmux resize-window")?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("tmux resize-window failed"))
        }
    }

    fn capture_pane(&self, id: &SessionId, lines: u16) -> Option<String> {
        let start = format!("-{}", lines);
        let output = Command::new("tmux")
            .args(["capture-pane", "-p", "-t", id.as_str(), "-S", &start])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
