use anyhow::Result;

use super::{MultiplexerBackend, Session, SessionId};

pub struct TmuxBackend;

impl TmuxBackend {
    pub fn new() -> Self {
        Self
    }
}

impl MultiplexerBackend for TmuxBackend {
    fn list_sessions(&self) -> Vec<Session> {
        // TODO: implement via `tmux list-sessions -F` and parse output
        vec![]
    }

    fn spawn_session(&self, _name: &str, _cmd: &str) -> Result<SessionId> {
        // TODO: implement via `tmux new-session -d -s <name> <cmd>`
        Ok(String::new())
    }

    fn attach_session(&self, _id: &SessionId) -> Result<()> {
        // TODO: implement via `tmux switch-client -t <id>`
        Ok(())
    }

    fn kill_session(&self, _id: &SessionId) -> Result<()> {
        // TODO: implement via `tmux kill-session -t <id>`
        Ok(())
    }

    fn resize_session(&self, _id: &SessionId, _rows: u16, _cols: u16) -> Result<()> {
        // TODO: implement via `tmux resize-window -t <id> -x <cols> -y <rows>`
        Ok(())
    }
}
