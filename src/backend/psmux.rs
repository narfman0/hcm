use anyhow::Result;

use super::{MultiplexerBackend, Session, SessionId, SpawnOptions};

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
        vec![]
    }

    fn spawn_session(&self, _opts: SpawnOptions) -> Result<SessionId> {
        Ok(String::new())
    }

    fn rename_session(&self, _id: &SessionId, _new_name: &str) -> Result<()> {
        Ok(())
    }

    fn attach_session(&self, _id: &SessionId) -> Result<()> {
        Ok(())
    }

    fn kill_session(&self, _id: &SessionId) -> Result<()> {
        Ok(())
    }

    fn resize_session(&self, _id: &SessionId, _rows: u16, _cols: u16) -> Result<()> {
        Ok(())
    }
}
