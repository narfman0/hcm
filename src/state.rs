use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::worktree::Worktree;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersistedState {
    #[serde(default)]
    pub sessions: HashMap<String, SessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub cwd: PathBuf,
    pub original_cmd: String,
    #[serde(default)]
    pub worktree: Option<Worktree>,
}

impl PersistedState {
    pub fn load(workspace: &Path) -> Self {
        let path = state_path(workspace);
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return Self::default(),
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    pub fn save(&self, workspace: &Path) -> Result<()> {
        std::fs::create_dir_all(workspace).context("failed to create hcm workspace dir")?;
        let path = state_path(workspace);
        let data = serde_json::to_string_pretty(self).context("failed to serialize state")?;
        std::fs::write(&path, data).context("failed to write state file")?;
        Ok(())
    }

    pub fn insert(&mut self, name: String, record: SessionRecord) {
        self.sessions.insert(name, record);
    }

    pub fn remove(&mut self, name: &str) -> Option<SessionRecord> {
        self.sessions.remove(name)
    }

    pub fn rename(&mut self, old: &str, new: String) {
        if let Some(record) = self.sessions.remove(old) {
            self.sessions.insert(new, record);
        }
    }

    pub fn get(&self, name: &str) -> Option<&SessionRecord> {
        self.sessions.get(name)
    }

    /// Drop records whose backend session no longer exists.
    pub fn gc(&mut self, live_names: &[String]) {
        self.sessions.retain(|name, _| live_names.iter().any(|n| n == name));
    }
}

fn state_path(workspace: &Path) -> PathBuf {
    workspace.join("state.json")
}
