use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, SystemTime};

use crate::backend::Session;

pub fn render(f: &mut Frame, session: &Session) {
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
            Constraint::Length(6), // session metadata
            Constraint::Min(1),    // placeholder for future terminal embed
            Constraint::Length(1), // footer
        ])
        .split(inner);

    render_metadata(f, chunks[0], session);
    render_placeholder(f, chunks[1]);
    render_footer(f, chunks[2]);
}

fn render_metadata(f: &mut Frame, area: ratatui::layout::Rect, session: &Session) {
    let status = if session.running { "running" } else { "idle" };
    let age = format_age(session.created_at);

    let lines = vec![
        Line::from(vec![
            Span::styled("  Name:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                session.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ID:      ", Style::default().fg(Color::DarkGray)),
            Span::raw(session.id.clone()),
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
        ]),
        Line::from(vec![
            Span::styled("  Started: ", Style::default().fg(Color::DarkGray)),
            Span::raw(age),
        ]),
    ];

    let meta = Paragraph::new(lines).block(
        Block::default()
            .title(" Session Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(meta, area);
}

fn render_placeholder(f: &mut Frame, area: ratatui::layout::Rect) {
    let placeholder = Paragraph::new(Line::from(Span::styled(
        "  [ Terminal embed — coming in a future phase ]",
        Style::default().fg(Color::DarkGray),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(placeholder, area);
}

fn render_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" / "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" back to dashboard"),
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
