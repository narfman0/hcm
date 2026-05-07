use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

use super::{MultiplexerBackend, Session, SessionId};

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
                "#{session_id}|#{session_name}|#{session_created}|#{session_activity}",
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

                Some(Session {
                    id,
                    name,
                    cmd: String::new(),
                    running: true,
                    created_at,
                })
            })
            .collect()
    }

    fn spawn_session(&self, name: &str, cmd: &str) -> Result<SessionId> {
        let mut args = vec!["new-session", "-d", "-s", name];
        let cmd_parts: Vec<&str> = cmd.split_whitespace().collect();
        args.extend(cmd_parts.iter().copied());

        Command::new("tmux")
            .args(&args)
            .status()
            .context("failed to run tmux new-session")?
            .success()
            .then_some(name.to_string())
            .ok_or_else(|| anyhow::anyhow!("tmux new-session failed"))
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
}
