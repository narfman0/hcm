use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::worktree::Worktree;

pub mod psmux;
pub mod tmux;
pub mod wezterm;

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

pub trait MultiplexerBackend {
    fn list_sessions(&self) -> Vec<Session>;
    fn spawn_session(&self, opts: SpawnOptions) -> Result<SessionId>;
    fn rename_session(&self, id: &SessionId, new_name: &str) -> Result<()>;
    fn attach_session(&self, id: &SessionId) -> Result<()>;
    fn kill_session(&self, id: &SessionId) -> Result<()>;
    fn resize_session(&self, id: &SessionId, rows: u16, cols: u16) -> Result<()>;
    /// Read recent terminal output for a session, if the backend supports it.
    /// Returns `None` if the backend can't snapshot.
    fn capture_pane(&self, _id: &SessionId, _lines: u16) -> Option<String> {
        None
    }
}

/// Auto-detect the best available backend at startup.
/// 1. WEZTERM_UNIX_SOCKET env set → WezTermBackend
/// 2. TMUX env set OR tmux in PATH → TmuxBackend
/// 3. Otherwise → PsmuxBackend
pub fn detect_backend() -> Box<dyn MultiplexerBackend> {
    if std::env::var("WEZTERM_UNIX_SOCKET").is_ok() {
        return Box::new(wezterm::WezTermBackend::new());
    }

    let has_tmux_env = std::env::var("TMUX").is_ok();
    let tmux_in_path = which_tmux();

    if has_tmux_env || tmux_in_path {
        return Box::new(tmux::TmuxBackend::new());
    }

    Box::new(psmux::PsmuxBackend::new())
}

fn which_tmux() -> bool {
    std::process::Command::new("which")
        .arg("tmux")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
