use anyhow::Result;

use super::{MultiplexerBackend, Session, SessionId};

/// Stub backend for Windows / fallback environments.
/// TODO: implement using psmux or Windows Terminal APIs.
pub struct PsmuxBackend;

impl PsmuxBackend {
    pub fn new() -> Self {
        Self
    }
}

impl MultiplexerBackend for PsmuxBackend {
    fn list_sessions(&self) -> Vec<Session> {
        // TODO: implement psmux session listing
        vec![]
    }

    fn spawn_session(&self, _name: &str, _cmd: &str) -> Result<SessionId> {
        // TODO: implement psmux session spawning
        Ok(String::new())
    }

    fn attach_session(&self, _id: &SessionId) -> Result<()> {
        // TODO: implement psmux session attach
        Ok(())
    }

    fn kill_session(&self, _id: &SessionId) -> Result<()> {
        // TODO: implement psmux session kill
        Ok(())
    }

    fn resize_session(&self, _id: &SessionId, _rows: u16, _cols: u16) -> Result<()> {
        // TODO: implement psmux session resize
        Ok(())
    }
}
