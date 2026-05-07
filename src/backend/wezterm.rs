use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;
use std::time::SystemTime;

use super::{MultiplexerBackend, Session, SessionId};

pub struct WezTermBackend;

impl WezTermBackend {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct WezPane {
    pane_id: u64,
    title: String,
    #[serde(default)]
    tty_name: String,
}

impl MultiplexerBackend for WezTermBackend {
    fn list_sessions(&self) -> Vec<Session> {
        let output = Command::new("wezterm")
            .args(["cli", "list", "--format", "json"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return vec![],
        };

        let panes: Vec<WezPane> = match serde_json::from_slice(&output.stdout) {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        panes
            .into_iter()
            .map(|p| Session {
                id: p.pane_id.to_string(),
                name: p.title.clone(),
                cmd: p.tty_name.clone(),
                running: true,
                created_at: SystemTime::now(),
            })
            .collect()
    }

    fn spawn_session(&self, name: &str, cmd: &str) -> Result<SessionId> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut c = Command::new("wezterm");
        c.args(["cli", "spawn", "--"]);
        c.args(&parts);

        let output = c.output().context("wezterm cli spawn failed")?;
        if !output.status.success() {
            anyhow::bail!(
                "wezterm spawn failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        // stdout contains the pane id
        let pane_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let _ = name; // WezTerm doesn't have named sessions like tmux; use pane id
        Ok(pane_id)
    }

    fn attach_session(&self, id: &SessionId) -> Result<()> {
        let output = Command::new("wezterm")
            .args(["cli", "activate-pane", "--pane-id", id])
            .output()
            .context("wezterm cli activate-pane failed")?;
        if !output.status.success() {
            anyhow::bail!(
                "wezterm activate-pane failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn kill_session(&self, id: &SessionId) -> Result<()> {
        // WezTerm doesn't have a direct kill-pane; send Ctrl-C then close
        Command::new("wezterm")
            .args(["cli", "send-text", "--pane-id", id, "--no-paste"])
            .arg("\x03")
            .output()
            .context("wezterm send-text failed")?;
        Ok(())
    }

    fn resize_session(&self, id: &SessionId, rows: u16, cols: u16) -> Result<()> {
        Command::new("wezterm")
            .args([
                "cli",
                "set-tab-title",
                "--pane-id",
                id,
                &format!("{}x{}", cols, rows),
            ])
            .output()
            .ok();
        Ok(())
    }
}
