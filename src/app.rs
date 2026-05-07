use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use std::time::Duration;

use crate::backend::{MultiplexerBackend, Session, SessionId};
use crate::ui;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Detail(SessionId),
}

pub struct App {
    pub view: View,
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub cmd: String,
    pub running: bool,
    backend: Box<dyn MultiplexerBackend>,
}

impl App {
    pub fn new(backend: Box<dyn MultiplexerBackend>, cmd: String) -> Self {
        Self {
            view: View::Dashboard,
            sessions: vec![],
            selected: 0,
            cmd,
            running: true,
            backend,
        }
    }

    pub fn refresh_sessions(&mut self) {
        self.sessions = self.backend.list_sessions();
        // clamp selection
        if !self.sessions.is_empty() && self.selected >= self.sessions.len() {
            self.selected = self.sessions.len() - 1;
        }
    }

    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        self.refresh_sessions();

        while self.running {
            terminal.draw(|f| {
                match &self.view.clone() {
                    View::Dashboard => {
                        ui::dashboard::render(f, &self.sessions, self.selected);
                    }
                    View::Detail(id) => {
                        let session = self.sessions.iter().find(|s| &s.id == id).cloned();
                        if let Some(s) = session {
                            ui::detail::render(f, &s);
                        } else {
                            // session gone — return to dashboard
                            self.view = View::Dashboard;
                        }
                    }
                }
            })?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match &self.view.clone() {
                        View::Dashboard => self.handle_dashboard_key(key.code),
                        View::Detail(_) => self.handle_detail_key(key.code),
                    }
                }
            } else {
                // periodic refresh
                self.refresh_sessions();
            }
        }
        Ok(())
    }

    fn handle_dashboard_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.sessions.is_empty() {
                    self.selected = (self.selected + 1).min(self.sessions.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(s) = self.sessions.get(self.selected) {
                    self.view = View::Detail(s.id.clone());
                }
            }
            KeyCode::Char('n') => {
                let name = format!("session-{}", self.sessions.len() + 1);
                let cmd = self.cmd.clone();
                let _ = self.backend.spawn_session(&name, &cmd);
                self.refresh_sessions();
            }
            KeyCode::Char('K') => {
                // uppercase K = kill selected session
                if let Some(s) = self.sessions.get(self.selected) {
                    let id = s.id.clone();
                    let _ = self.backend.kill_session(&id);
                    self.refresh_sessions();
                }
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = View::Dashboard;
            }
            _ => {}
        }
    }
}
