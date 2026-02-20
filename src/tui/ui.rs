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

            // State indicator with ignored overlay
            let (indicator, indicator_color, prefix_color) = if project.ignored {
                ("⊘", Color::DarkGray, Color::DarkGray)
            } else {
                match project.state {
                    ProjectState::Active => ("●", Color::Green, Color::Green),
                    ProjectState::Inactive => ("○", Color::Yellow, Color::Yellow),
                    ProjectState::Archived => ("◌", Color::DarkGray, Color::DarkGray),
                }
            };

            let prefix = if is_selected { "▸ " } else { "  " };

            // Calculate padding for right-alignment
            let name_len = project.name.len() as u16;
            let prefix_len = 2u16; // "▸ " or "  "
            let indicator_len = 2u16; // "● "
            let available = area
                .width
                .saturating_sub(prefix_len + name_len + indicator_len);
            let padding = " ".repeat(available as usize);

            // Style the name based on selection and state
            let name_style = if is_selected {
                if project.ignored {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                }
            } else if project.ignored {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Reset)
            };

            let prefix_style = if is_selected {
                Style::default().fg(prefix_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(&project.name, name_style),
                Span::raw(padding),
                Span::styled(indicator, Style::default().fg(indicator_color)),
            ]);

            let style = if is_selected {
                Style::default().bg(Color::Rgb(40, 44, 52)) // Subtle dark blue-gray
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
        // Build meta line with colored components
        let mut meta_spans = vec![
            Span::styled("Last touched ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_relative_time(project.last_touched),
                Style::default().fg(Color::Rgb(150, 150, 150)),
            ),
            Span::styled(" · ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", project.visit_count),
                Style::default().fg(Color::Rgb(150, 150, 150)),
            ),
            Span::styled(" visits", Style::default().fg(Color::DarkGray)),
        ];

        if let Some(ref origin) = project.git_origin {
            // Extract repo name from origin URL for compact display
            let repo_name = origin
                .rsplit('/')
                .next()
                .unwrap_or(origin)
                .trim_end_matches(".git");
            meta_spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
            meta_spans.push(Span::styled("⎇ ", Style::default().fg(Color::Green)));
            meta_spans.push(Span::styled(
                repo_name,
                Style::default().fg(Color::Rgb(120, 180, 120)),
            ));
        }

        let text = vec![
            Line::from("─".repeat(area.width as usize))
                .style(Style::default().fg(Color::Rgb(60, 60, 60))),
            Line::from(Span::styled(
                project.path.display().to_string(),
                Style::default().fg(Color::Rgb(100, 200, 220)),
            )),
            Line::from(meta_spans),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    // Build status line with colored components
    let mut status_spans = vec![Span::raw("  ")];

    // Match count with color based on results
    let match_color = if app.filtered.is_empty() && !app.input.is_empty() {
        Color::Red // No matches
    } else if app.filtered.len() < app.projects.len() / 4 && !app.input.is_empty() {
        Color::Yellow // Few matches
    } else {
        Color::Green // Good matches
    };

    status_spans.push(Span::styled(
        format!("{}", app.filtered.len()),
        Style::default().fg(match_color),
    ));
    status_spans.push(Span::styled(
        format!("/{}", app.projects.len()),
        Style::default().fg(Color::DarkGray),
    ));

    if app.show_ignored {
        status_spans.push(Span::styled(
            " [showing ignored]",
            Style::default().fg(Color::Rgb(180, 140, 100)),
        ));
    }

    // Help hints with colored command keys
    if app.command_mode {
        status_spans.push(Span::styled("  Command: ", Style::default().fg(Color::Magenta)));
        for (i, cmd) in ["c:config", "a:archive", "d:deactivate", "i:ignore", "I:show-ignored"]
            .iter()
            .enumerate()
        {
            if i > 0 {
                status_spans.push(Span::raw(" "));
            }
            let parts: Vec<&str> = cmd.split(':').collect();
            status_spans.push(Span::styled(parts[0], Style::default().fg(Color::Magenta)));
            status_spans.push(Span::styled(":", Style::default().fg(Color::DarkGray)));
            status_spans.push(Span::styled(parts[1], Style::default().fg(Color::DarkGray)));
        }
        status_spans.push(Span::raw(" "));
        status_spans.push(Span::styled("ESC", Style::default().fg(Color::Magenta)));
        status_spans.push(Span::styled(":cancel", Style::default().fg(Color::DarkGray)));
    } else if app.input.is_empty() {
        status_spans.push(Span::styled("  /", Style::default().fg(Color::Cyan)));
        status_spans.push(Span::styled(":command ", Style::default().fg(Color::DarkGray)));
        status_spans.push(Span::styled("?", Style::default().fg(Color::Cyan)));
        status_spans.push(Span::styled(":help", Style::default().fg(Color::DarkGray)));
    }

    // Calculate separator length
    let text_len: usize = status_spans.iter().map(|s| s.content.len()).sum();
    let separator_len = area.width.saturating_sub(text_len as u16);
    status_spans.push(Span::styled(
        format!(" {}", "─".repeat(separator_len as usize)),
        Style::default().fg(Color::Rgb(60, 60, 60)),
    ));

    // Input line with prompt
    let (prompt, prompt_color, input_text) = if app.command_mode {
        ("/", Color::Magenta, &app.command_input)
    } else {
        (">", Color::Cyan, &app.input)
    };

    let text = vec![
        Line::from(status_spans),
        Line::from(vec![
            Span::styled(
                format!("  {} ", prompt),
                Style::default().fg(prompt_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(input_text, Style::default().fg(Color::White)),
            Span::styled(
                "_",
                Style::default()
                    .fg(prompt_color)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
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
        Line::from(Span::styled(
            help::HELP_TITLE,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (key, desc) in help::KEYBINDINGS {
        if key.is_empty() {
            // Empty line or section header
            if desc.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:18}", key),
                    Style::default().fg(Color::Rgb(100, 200, 220)),
                ),
                Span::styled(*desc, Style::default().fg(Color::Rgb(200, 200, 200))),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::styled(" to close", Style::default().fg(Color::DarkGray)),
    ]));

    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 200, 220)))
        .title(" Help ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

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
        Line::from(Span::styled(
            "Settings",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
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
            Span::styled("  Projects directory: ", Style::default().fg(Color::Rgb(150, 150, 150))),
            Span::styled(tracked, Style::default().fg(Color::Rgb(180, 140, 200))),
        ]));

        // Ignore patterns count
        let pattern_count = cfg.ignore_patterns.len();
        let pattern_color = if pattern_count > 0 {
            Color::Rgb(180, 140, 200)
        } else {
            Color::DarkGray
        };
        lines.push(Line::from(vec![
            Span::styled("  Ignore patterns:    ", Style::default().fg(Color::Rgb(150, 150, 150))),
            Span::styled(format!("{} patterns", pattern_count), Style::default().fg(pattern_color)),
        ]));

        // Config file path
        if let Ok(config_path) = crate::config::paths::get_config_file() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Config file: ", Style::default().fg(Color::Rgb(150, 150, 150))),
                Span::styled(
                    config_path.display().to_string(),
                    Style::default().fg(Color::Rgb(120, 120, 120)),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("  Failed to load configuration", Style::default().fg(Color::Red)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("e", Style::default().fg(Color::Magenta)),
        Span::styled(" to edit in $EDITOR, ", Style::default().fg(Color::DarkGray)),
        Span::styled("c", Style::default().fg(Color::Magenta)),
        Span::styled(" to close", Style::default().fg(Color::DarkGray)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(180, 140, 200)))
        .title(" Settings ")
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );

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
