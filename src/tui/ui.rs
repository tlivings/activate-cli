use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use crate::database::models::ProjectState;
use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: list (flexible), status (3 lines), input (2 lines)
    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .split(area);

    render_list(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let project = &app.projects[idx];
            let is_selected = i == app.selected;

            let prefix = if is_selected { "▸ " } else { "  " };
            let state_indicator = match project.state {
                ProjectState::Active => ("●", Color::Green),
                ProjectState::Inactive => ("○", Color::Yellow),
                ProjectState::Archived => ("◌", Color::DarkGray),
            };

            // Calculate padding for right-alignment
            let name_len = project.name.len() as u16;
            let prefix_len = 2u16; // "▸ " or "  "
            let indicator_len = 2u16; // "● "
            let available = area.width.saturating_sub(prefix_len + name_len + indicator_len);
            let padding = " ".repeat(available as usize);

            let line = Line::from(vec![
                Span::raw(prefix),
                Span::raw(&project.name),
                Span::raw(padding),
                Span::styled(state_indicator.0, Style::default().fg(state_indicator.1)),
            ]);

            let style = if is_selected {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(project) = app.selected_project() {
        let path_line = format!("{}", project.path.display());
        let meta_line = format!(
            "Last touched {} · {} visits",
            format_relative_time(project.last_touched),
            project.visit_count
        );

        let text = vec![
            Line::from("─".repeat(area.width as usize))
                .style(Style::default().fg(Color::DarkGray)),
            Line::from(path_line).style(Style::default().fg(Color::Cyan)),
            Line::from(meta_line).style(Style::default().fg(Color::DarkGray)),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let match_info = format!("{}/{}", app.filtered.len(), app.projects.len());
    let separator_len = area.width.saturating_sub(match_info.len() as u16 + 4);
    let separator = format!(
        "  {} {}",
        match_info,
        "─".repeat(separator_len as usize)
    );

    let text = vec![
        Line::from(separator).style(Style::default().fg(Color::DarkGray)),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.input),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_hours() < 1 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_days() < 1 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_weeks() < 1 {
        format!("{}d ago", diff.num_days())
    } else {
        format!("{}w ago", diff.num_weeks())
    }
}
