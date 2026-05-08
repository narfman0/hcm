use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crate::backend::{MultiplexerBackend, Session, SessionId, SpawnOptions};
use crate::gh;
use crate::state::{PersistedState, SessionRecord};
use crate::ui;
use crate::worktree;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Detail(SessionId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewSessionField {
    Name,
    Cmd,
    Cwd,
    Worktree,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    NewSession {
        name: String,
        cmd: String,
        cwd: String,
        use_worktree: bool,
        focus: NewSessionField,
    },
    Rename {
        target: SessionId,
        target_name: String,
        buffer: String,
    },
    ConfirmKill {
        target: SessionId,
        target_name: String,
    },
    GhPanel {
        title: String,
        content: String,
        scroll: u16,
    },
}

pub struct App {
    pub view: View,
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub cmd: String,
    pub running: bool,
    pub overlay: Option<Overlay>,
    pub workspace: PathBuf,
    pub cwd: PathBuf,
    pub snapshot: String,
    backend: Box<dyn MultiplexerBackend>,
    state: PersistedState,
    pending_attach: Option<SessionId>,
}

impl App {
    pub fn new(
        backend: Box<dyn MultiplexerBackend>,
        cmd: String,
        workspace: PathBuf,
        cwd: PathBuf,
    ) -> Self {
        let state = PersistedState::load(&workspace);
        Self {
            view: View::Dashboard,
            sessions: vec![],
            selected: 0,
            cmd,
            running: true,
            overlay: None,
            workspace,
            cwd,
            snapshot: String::new(),
            backend,
            state,
            pending_attach: None,
        }
    }

    pub fn refresh_sessions(&mut self) {
        let mut sessions = self.backend.list_sessions();
        for s in sessions.iter_mut() {
            if let Some(rec) = self.state.get(&s.name) {
                s.cwd = rec.cwd.clone();
                if !rec.original_cmd.is_empty() {
                    s.cmd = rec.original_cmd.clone();
                }
                s.worktree = rec.worktree.clone();
            }
        }
        self.sessions = sessions;
        let live: Vec<String> = self.sessions.iter().map(|s| s.name.clone()).collect();
        self.state.gc(&live);
        let _ = self.state.save(&self.workspace);

        if !self.sessions.is_empty() && self.selected >= self.sessions.len() {
            self.selected = self.sessions.len() - 1;
        }
    }

    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        self.refresh_sessions();

        while self.running {
            self.refresh_snapshot();

            terminal.draw(|f| {
                let area = f.area();
                match &self.view.clone() {
                    View::Dashboard => {
                        ui::dashboard::render(f, &self.sessions, self.selected);
                    }
                    View::Detail(id) => {
                        let session = self.sessions.iter().find(|s| &s.id == id).cloned();
                        if let Some(s) = session {
                            ui::detail::render(f, &s, &self.snapshot);
                        } else {
                            self.view = View::Dashboard;
                        }
                    }
                }
                if let Some(overlay) = &self.overlay {
                    ui::modal::render(f, area, overlay);
                }
            })?;

            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key);
                }
            } else {
                self.refresh_sessions();
            }

            if let Some(id) = self.pending_attach.take() {
                self.suspend_and_attach(terminal, &id)?;
                self.refresh_sessions();
            }
        }
        Ok(())
    }

    fn refresh_snapshot(&mut self) {
        if let View::Detail(id) = &self.view.clone() {
            self.snapshot = self
                .backend
                .capture_pane(id, 200)
                .unwrap_or_else(|| String::from("(no snapshot available)"));
        } else {
            self.snapshot.clear();
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.overlay.is_some() {
            self.handle_overlay_key(key);
            return;
        }
        match self.view.clone() {
            View::Dashboard => self.handle_dashboard_key(key),
            View::Detail(_) => self.handle_detail_key(key),
        }
    }

    fn handle_dashboard_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('Q') => self.running = false,
            KeyCode::Down => {
                if !self.sessions.is_empty() {
                    self.selected = (self.selected + 1).min(self.sessions.len() - 1);
                }
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(s) = self.sessions.get(self.selected) {
                    self.view = View::Detail(s.id.clone());
                }
            }
            KeyCode::Char('n') if !ctrl => self.quick_new(),
            KeyCode::Char('N') => self.open_new_session_modal(),
            KeyCode::Char('r') if ctrl => self.open_rename_modal(),
            KeyCode::Char('k') if !ctrl => self.open_kill_confirm(),
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let id = match &self.view {
            View::Detail(id) => id.clone(),
            _ => return,
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = View::Dashboard;
            }
            KeyCode::Char('t') if ctrl => {
                self.pending_attach = Some(id);
            }
            KeyCode::Char('y') if ctrl => {
                self.open_gh_panel(&id);
            }
            KeyCode::Char('r') if ctrl => {
                self.open_rename_modal_for(&id);
            }
            KeyCode::Char('\\') if ctrl => {
                self.view = View::Dashboard;
            }
            _ => {}
        }
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) {
        let overlay = match self.overlay.take() {
            Some(o) => o,
            None => return,
        };
        match overlay {
            Overlay::NewSession {
                mut name,
                mut cmd,
                mut cwd,
                mut use_worktree,
                mut focus,
            } => match key.code {
                KeyCode::Esc => {}
                KeyCode::Tab => {
                    focus = next_field(&focus);
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
                KeyCode::BackTab => {
                    focus = prev_field(&focus);
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
                KeyCode::Enter => {
                    let cwd_path = PathBuf::from(shellexpand_home(&cwd));
                    self.spawn_with(&name, &cmd, &cwd_path, use_worktree);
                }
                KeyCode::Char(' ') if focus == NewSessionField::Worktree => {
                    use_worktree = !use_worktree;
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
                KeyCode::Backspace => {
                    match focus {
                        NewSessionField::Name => {
                            name.pop();
                        }
                        NewSessionField::Cmd => {
                            cmd.pop();
                        }
                        NewSessionField::Cwd => {
                            cwd.pop();
                        }
                        NewSessionField::Worktree => {}
                    }
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
                KeyCode::Char(c) => {
                    match focus {
                        NewSessionField::Name => name.push(c),
                        NewSessionField::Cmd => cmd.push(c),
                        NewSessionField::Cwd => cwd.push(c),
                        NewSessionField::Worktree => {
                            if c == 'y' || c == 'Y' {
                                use_worktree = true;
                            } else if c == 'n' || c == 'N' {
                                use_worktree = false;
                            }
                        }
                    }
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
                _ => {
                    self.overlay = Some(Overlay::NewSession {
                        name,
                        cmd,
                        cwd,
                        use_worktree,
                        focus,
                    });
                }
            },
            Overlay::Rename {
                target,
                target_name,
                mut buffer,
            } => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    if !buffer.is_empty() && buffer != target_name {
                        let _ = self.backend.rename_session(&target, &buffer);
                        self.state.rename(&target_name, buffer.clone());
                        let _ = self.state.save(&self.workspace);
                        if let View::Detail(ref mut id) = self.view {
                            if id == &target {
                                *id = buffer.clone();
                            }
                        }
                    }
                    self.refresh_sessions();
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    self.overlay = Some(Overlay::Rename {
                        target,
                        target_name,
                        buffer,
                    });
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                    self.overlay = Some(Overlay::Rename {
                        target,
                        target_name,
                        buffer,
                    });
                }
                _ => {
                    self.overlay = Some(Overlay::Rename {
                        target,
                        target_name,
                        buffer,
                    });
                }
            },
            Overlay::ConfirmKill {
                target,
                target_name,
            } => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.do_kill(&target, &target_name);
                }
                _ => {}
            },
            Overlay::GhPanel {
                title,
                content,
                mut scroll,
            } => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    scroll = scroll.saturating_add(1);
                    self.overlay = Some(Overlay::GhPanel {
                        title,
                        content,
                        scroll,
                    });
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    scroll = scroll.saturating_sub(1);
                    self.overlay = Some(Overlay::GhPanel {
                        title,
                        content,
                        scroll,
                    });
                }
                _ => {}
            },
        }
    }

    fn quick_new(&mut self) {
        let name = self.next_session_name();
        let cmd = self.cmd.clone();
        let cwd = self.cwd.clone();
        self.spawn_with(&name, &cmd, &cwd, true);
    }

    fn next_session_name(&self) -> String {
        for i in 1..1000 {
            let candidate = format!("session-{}", i);
            if !self.sessions.iter().any(|s| s.name == candidate) {
                return candidate;
            }
        }
        format!("session-{}", self.sessions.len() + 1)
    }

    fn spawn_with(&mut self, name: &str, cmd: &str, cwd: &PathBuf, use_worktree: bool) {
        if name.is_empty() || cmd.is_empty() {
            return;
        }
        let mut spawn_cwd = cwd.clone();
        let mut wt = None;

        if use_worktree {
            if let Some(repo_root) = worktree::find_repo_root(cwd) {
                match worktree::create(&self.workspace, &repo_root, name) {
                    Ok(w) => {
                        spawn_cwd = w.path.clone();
                        wt = Some(w);
                    }
                    Err(_) => {
                        // worktree failed — fall through and spawn in plain cwd
                    }
                }
            }
        }

        let opts = SpawnOptions {
            name,
            cmd,
            cwd: &spawn_cwd,
        };

        if self.backend.spawn_session(opts).is_ok() {
            self.state.insert(
                name.to_string(),
                SessionRecord {
                    cwd: spawn_cwd,
                    original_cmd: cmd.to_string(),
                    worktree: wt,
                },
            );
            let _ = self.state.save(&self.workspace);
            self.refresh_sessions();
        } else if let Some(w) = wt {
            // backend spawn failed — clean up the worktree we just made
            let _ = worktree::remove(&w);
        }
    }

    fn open_new_session_modal(&mut self) {
        let name = self.next_session_name();
        self.overlay = Some(Overlay::NewSession {
            name,
            cmd: self.cmd.clone(),
            cwd: self.cwd.to_string_lossy().into_owned(),
            use_worktree: true,
            focus: NewSessionField::Name,
        });
    }

    fn open_rename_modal(&mut self) {
        if let Some(s) = self.sessions.get(self.selected) {
            self.overlay = Some(Overlay::Rename {
                target: s.id.clone(),
                target_name: s.name.clone(),
                buffer: s.name.clone(),
            });
        }
    }

    fn open_rename_modal_for(&mut self, id: &SessionId) {
        if let Some(s) = self.sessions.iter().find(|s| &s.id == id) {
            self.overlay = Some(Overlay::Rename {
                target: s.id.clone(),
                target_name: s.name.clone(),
                buffer: s.name.clone(),
            });
        }
    }

    fn open_kill_confirm(&mut self) {
        if let Some(s) = self.sessions.get(self.selected) {
            self.overlay = Some(Overlay::ConfirmKill {
                target: s.id.clone(),
                target_name: s.name.clone(),
            });
        }
    }

    fn open_gh_panel(&mut self, id: &SessionId) {
        let session = match self.sessions.iter().find(|s| &s.id == id) {
            Some(s) => s.clone(),
            None => return,
        };
        let (repo_root, branch): (PathBuf, Option<String>) = if let Some(wt) = &session.worktree {
            (wt.repo_root.clone(), Some(wt.branch.clone()))
        } else if let Some(root) = worktree::find_repo_root(&session.cwd) {
            (root, None)
        } else {
            self.overlay = Some(Overlay::GhPanel {
                title: "GitHub CLI".to_string(),
                content: "Session is not in a git repository.".to_string(),
                scroll: 0,
            });
            return;
        };
        let result = gh::pr_status(&repo_root, branch.as_deref());
        self.overlay = Some(Overlay::GhPanel {
            title: result.title,
            content: result.body,
            scroll: 0,
        });
    }

    fn do_kill(&mut self, id: &SessionId, name: &str) {
        let _ = self.backend.kill_session(id);
        if let Some(record) = self.state.remove(name) {
            if let Some(wt) = record.worktree {
                let _ = worktree::remove(&wt);
            }
        }
        let _ = self.state.save(&self.workspace);
        if matches!(&self.view, View::Detail(view_id) if view_id == id) {
            self.view = View::Dashboard;
        }
        self.refresh_sessions();
    }

    fn suspend_and_attach(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        id: &SessionId,
    ) -> Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        let _ = self.backend.attach_session(id);

        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        terminal.clear()?;
        Ok(())
    }
}

fn next_field(f: &NewSessionField) -> NewSessionField {
    match f {
        NewSessionField::Name => NewSessionField::Cmd,
        NewSessionField::Cmd => NewSessionField::Cwd,
        NewSessionField::Cwd => NewSessionField::Worktree,
        NewSessionField::Worktree => NewSessionField::Name,
    }
}

fn prev_field(f: &NewSessionField) -> NewSessionField {
    match f {
        NewSessionField::Name => NewSessionField::Worktree,
        NewSessionField::Cmd => NewSessionField::Name,
        NewSessionField::Cwd => NewSessionField::Cmd,
        NewSessionField::Worktree => NewSessionField::Cwd,
    }
}

fn shellexpand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    } else if s == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().into_owned();
        }
    }
    s.to_string()
}
