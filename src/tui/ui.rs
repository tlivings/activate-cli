use crate::database::models::ProjectState;
use crate::tui::app::App;
use crate::tui::help;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

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

    // Render overlays on top if active
    if app.show_help {
        render_help_overlay(frame, area);
    } else if app.show_settings {
        render_settings_overlay(frame, area);
    }
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

            // State indicator with ignored overlay
            let (indicator, color) = if project.ignored {
                ("⊘", Color::DarkGray) // Ignored indicator
            } else {
                match project.state {
                    ProjectState::Active => ("●", Color::Green),
                    ProjectState::Inactive => ("○", Color::Yellow),
                    ProjectState::Archived => ("◌", Color::DarkGray),
                }
            };

            // Calculate padding for right-alignment
            let name_len = project.name.len() as u16;
            let prefix_len = 2u16; // "▸ " or "  "
            let indicator_len = 2u16; // "● "
            let available = area
                .width
                .saturating_sub(prefix_len + name_len + indicator_len);
            let padding = " ".repeat(available as usize);

            // Dim the name if ignored
            let name_style = if project.ignored {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(&project.name, name_style),
                Span::raw(padding),
                Span::styled(indicator, Style::default().fg(color)),
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

        // Build meta line with optional git origin
        let mut meta_parts = vec![
            format!(
                "Last touched {}",
                format_relative_time(project.last_touched)
            ),
            format!("{} visits", project.visit_count),
        ];
        if let Some(ref origin) = project.git_origin {
            // Extract repo name from origin URL for compact display
            let repo_name = origin
                .rsplit('/')
                .next()
                .unwrap_or(origin)
                .trim_end_matches(".git");
            meta_parts.push(format!("⎇ {}", repo_name));
        }
        let meta_line = meta_parts.join(" · ");

        let text = vec![
            Line::from("─".repeat(area.width as usize)).style(Style::default().fg(Color::DarkGray)),
            Line::from(path_line).style(Style::default().fg(Color::Cyan)),
            Line::from(meta_line).style(Style::default().fg(Color::DarkGray)),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let match_info = format!("{}/{}", app.filtered.len(), app.projects.len());
    let ignored_info = if app.show_ignored {
        " [showing ignored]"
    } else {
        ""
    };
    let help_hint = if app.command_mode {
        "  Command: c:config a:archive d:deactivate i:ignore I:show-ignored ESC:cancel"
    } else if app.input.is_empty() {
        "  /:command ?:help"
    } else {
        ""
    };

    let info_str = format!("{}{}{}", match_info, ignored_info, help_hint);
    let separator_len = area.width.saturating_sub(info_str.len() as u16 + 4);
    let separator = format!("  {} {}", info_str, "─".repeat(separator_len as usize));

    let (prompt, input_text) = if app.command_mode {
        ("/", &app.command_input)
    } else {
        (">", &app.input)
    };

    let text = vec![
        Line::from(separator).style(Style::default().fg(Color::DarkGray)),
        Line::from(vec![
            Span::styled(format!("  {} ", prompt), Style::default().fg(Color::Cyan)),
            Span::raw(input_text),
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

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Calculate centered area (50% width, 60% height, min 40x15)
    let help_area = centered_rect(50, 60, area);

    // Clear the area first (important for overlay effect)
    frame.render_widget(Clear, help_area);

    // Build help content
    let mut lines = vec![
        Line::from(help::HELP_TITLE).style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
    ];

    for (key, desc) in help::KEYBINDINGS {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:18}", key), Style::default().fg(Color::Cyan)),
            Span::raw(*desc),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Press ? to close").style(Style::default().fg(Color::DarkGray)));

    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help ")
        .title_style(Style::default().add_modifier(Modifier::BOLD));

    let help_paragraph = Paragraph::new(lines)
        .block(help_block)
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(help_paragraph, help_area);
}

fn render_settings_overlay(frame: &mut Frame, area: Rect) {
    use crate::config::Config;

    // Load current config
    let config = Config::load().ok();

    let settings_area = centered_rect(60, 50, area);
    frame.render_widget(Clear, settings_area);

    let mut lines = vec![
        Line::from("Settings").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
    ];

    if let Some(ref cfg) = config {
        // Tracked directory
        let tracked = cfg
            .tracked_directory
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not set)".to_string());
        lines.push(Line::from(vec![
            Span::styled("  Projects directory: ", Style::default().fg(Color::DarkGray)),
            Span::styled(tracked, Style::default().fg(Color::Cyan)),
        ]));

        // Ignore patterns count
        let pattern_count = cfg.ignore_patterns.len();
        lines.push(Line::from(vec![
            Span::styled("  Ignore patterns:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} patterns", pattern_count), Style::default()),
        ]));

        // Config file path
        if let Ok(config_path) = crate::config::paths::get_config_file() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Config file: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    config_path.display().to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        lines.push(Line::from("  Failed to load configuration")
            .style(Style::default().fg(Color::Red)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(
        Line::from("Press 'e' to edit in $EDITOR, 'c' to close")
            .style(Style::default().fg(Color::DarkGray)),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(" Settings ")
        .title_style(Style::default().add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, settings_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = (area.width * percent_x / 100).max(40).min(area.width);
    let height = (area.height * percent_y / 100).max(15).min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
