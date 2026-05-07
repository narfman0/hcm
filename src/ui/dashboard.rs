use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::{Duration, SystemTime};

use crate::backend::Session;

pub fn render(f: &mut Frame, sessions: &[Session], selected: usize) {
    let area = f.area();

    let outer_block = Block::default()
        .title(" hcm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar
            Constraint::Min(1),    // session list
            Constraint::Length(1), // footer / hints
        ])
        .split(inner);

    render_header(f, chunks[0]);
    render_sessions(f, chunks[1], sessions, selected);
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  Sessions", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("                          "),
        Span::styled("[n]", Style::default().fg(Color::Yellow)),
        Span::raw("ew  "),
        Span::styled("[Q]", Style::default().fg(Color::Yellow)),
        Span::raw("uit"),
    ]));
    f.render_widget(header, area);
}

fn render_sessions(f: &mut Frame, area: Rect, sessions: &[Session], selected: usize) {
    let items: Vec<ListItem> = if sessions.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (no sessions — press [n] to start one)",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let marker = if i == selected { "▶ " } else { "  " };
                let status = if s.running { "running" } else { "idle" };
                let age = format_age(s.created_at);
                let line = Line::from(vec![
                    Span::styled(
                        marker,
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        format!("{:<22}", s.name),
                        Style::default().add_modifier(if i == selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                    ),
                    Span::styled(
                        format!("{:<10}", short_cmd(&s.cmd)),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(
                        format!("{:<10}", status),
                        Style::default().fg(if s.running { Color::Green } else { Color::Yellow }),
                    ),
                    Span::styled(age, Style::default().fg(Color::DarkGray)),
                ]);
                let style = if i == selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(line).style(style)
            })
            .collect()
    };

    let mut state = ListState::default();
    if !sessions.is_empty() {
        state.select(Some(selected));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_stateful_widget(list, area, &mut state);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" attach  "),
        Span::styled("[K]", Style::default().fg(Color::Yellow)),
        Span::raw(" kill  "),
        Span::styled("[n]", Style::default().fg(Color::Yellow)),
        Span::raw(" new  "),
        Span::styled("[↑↓/jk]", Style::default().fg(Color::Yellow)),
        Span::raw(" navigate  "),
        Span::styled("[Q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]));
    f.render_widget(footer, area);
}

fn format_age(created_at: SystemTime) -> String {
    let elapsed = SystemTime::now()
        .duration_since(created_at)
        .unwrap_or(Duration::ZERO);
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

fn short_cmd(cmd: &str) -> &str {
    cmd.split_whitespace().next().unwrap_or(cmd)
}
