use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{NewSessionField, Overlay};

pub fn render(f: &mut Frame, area: Rect, overlay: &Overlay) {
    match overlay {
        Overlay::NewSession {
            name,
            cmd,
            cwd,
            use_worktree,
            focus,
        } => render_new_session(f, area, name, cmd, cwd, *use_worktree, focus),
        Overlay::Rename {
            target_name,
            buffer,
            ..
        } => render_rename(f, area, target_name, buffer),
        Overlay::ConfirmKill { target_name, .. } => render_confirm_kill(f, area, target_name),
        Overlay::GhPanel {
            title,
            content,
            scroll,
        } => render_gh_panel(f, area, title, content, *scroll),
    }
}

pub fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width.saturating_mul(percent_x) / 100;
    let popup_width = popup_width.max(40).min(area.width.saturating_sub(2));
    let popup_height = height.min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    }
}

fn render_new_session(
    f: &mut Frame,
    area: Rect,
    name: &str,
    cmd: &str,
    cwd: &str,
    use_worktree: bool,
    focus: &NewSessionField,
) {
    let popup = centered_rect(70, 14, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(" New session ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    f.render_widget(field_paragraph("Name:    ", name, focus == &NewSessionField::Name), chunks[0]);
    f.render_widget(field_paragraph("Command: ", cmd, focus == &NewSessionField::Cmd), chunks[1]);
    f.render_widget(field_paragraph("Cwd:     ", cwd, focus == &NewSessionField::Cwd), chunks[2]);

    let wt_value = if use_worktree { "[x] yes" } else { "[ ] no" };
    f.render_widget(
        field_paragraph(
            "Worktree:",
            wt_value,
            focus == &NewSessionField::Worktree,
        ),
        chunks[3],
    );

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" next  "),
        Span::styled("[Space/y/n]", Style::default().fg(Color::Yellow)),
        Span::raw(" toggle  "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" submit  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]));
    f.render_widget(footer, chunks[5]);
}

fn field_paragraph<'a>(label: &'a str, value: &'a str, focused: bool) -> Paragraph<'a> {
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = if focused {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let cursor = if focused { "_" } else { "" };
    Paragraph::new(Line::from(vec![
        Span::styled(label, label_style),
        Span::raw(" "),
        Span::styled(value.to_string(), value_style),
        Span::styled(cursor, Style::default().fg(Color::Yellow)),
    ]))
}

fn render_rename(f: &mut Frame, area: Rect, target_name: &str, buffer: &str) {
    let popup = centered_rect(60, 6, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" Rename — {} ", target_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(inner);

    f.render_widget(field_paragraph("New name:", buffer, true), chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" confirm  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]));
    f.render_widget(footer, chunks[1]);
}

fn render_confirm_kill(f: &mut Frame, area: Rect, target_name: &str) {
    let popup = centered_rect(50, 6, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Kill session? ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let body = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Kill "),
            Span::styled(
                target_name.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("? Worktree (if any) will be removed; branch kept."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y/Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" yes  "),
            Span::styled("[any]", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]),
    ])
    .wrap(Wrap { trim: false });
    f.render_widget(body, inner);
}

fn render_gh_panel(f: &mut Frame, area: Rect, title: &str, content: &str, scroll: u16) {
    let popup = centered_rect(80, area.height.saturating_sub(4), area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let body = Paragraph::new(content.to_string())
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(body, chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Yellow)),
        Span::raw(" scroll  "),
        Span::styled("[any other]", Style::default().fg(Color::Yellow)),
        Span::raw(" close"),
    ]));
    f.render_widget(footer, chunks[1]);
}
