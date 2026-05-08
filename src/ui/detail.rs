use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, SystemTime};

use crate::tmux::Session;

pub fn render(f: &mut Frame, session: &Session, snapshot: &str) {
    let area = f.area();

    let outer_block = Block::default()
        .title(format!(" hcm — {} ", session.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_metadata(f, chunks[0], session);
    render_snapshot(f, chunks[1], snapshot);
    render_footer(f, chunks[2]);
}

fn render_metadata(f: &mut Frame, area: ratatui::layout::Rect, session: &Session) {
    let status = if session.running { "running" } else { "idle" };
    let age = format_age(session.created_at);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  Name:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                session.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Command: ", Style::default().fg(Color::DarkGray)),
            Span::styled(session.cmd.clone(), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("  Status:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                status,
                Style::default().fg(if session.running {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
            Span::raw("    "),
            Span::styled("started ", Style::default().fg(Color::DarkGray)),
            Span::raw(age),
        ]),
        Line::from(vec![
            Span::styled("  Cwd:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(session.cwd.to_string_lossy().into_owned()),
        ]),
    ];

    if let Some(wt) = &session.worktree {
        lines.push(Line::from(vec![
            Span::styled("  Worktree: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                wt.path.to_string_lossy().into_owned(),
                Style::default().fg(Color::Green),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Branch:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(wt.branch.clone(), Style::default().fg(Color::Magenta)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Repo:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                wt.repo_root
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  Worktree: ", Style::default().fg(Color::DarkGray)),
            Span::styled("—", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let meta = Paragraph::new(lines).block(
        Block::default()
            .title(" Session Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(meta, area);
}

fn render_snapshot(f: &mut Frame, area: ratatui::layout::Rect, snapshot: &str) {
    let block = Block::default()
        .title(" Output (read-only snapshot) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let lines: Vec<&str> = snapshot.lines().collect();
    let visible: Vec<Line> = if lines.len() > height {
        lines[lines.len() - height..]
            .iter()
            .map(|l| Line::raw(l.to_string()))
            .collect()
    } else {
        lines.iter().map(|l| Line::raw(l.to_string())).collect()
    };

    let paragraph = Paragraph::new(visible).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Ctrl-T]", Style::default().fg(Color::Yellow)),
        Span::raw(" terminal  "),
        Span::styled("[Ctrl-Y]", Style::default().fg(Color::Yellow)),
        Span::raw(" gh  "),
        Span::styled("[Ctrl-R]", Style::default().fg(Color::Yellow)),
        Span::raw(" rename  "),
        Span::styled("[Ctrl-\\]", Style::default().fg(Color::Yellow)),
        Span::raw(" detach  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
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
