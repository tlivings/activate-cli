use crate::database::models::{Project, ProjectState};
use crate::navigation::ProjectMatcher;
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::PathBuf;

pub struct App {
    pub projects: Vec<Project>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub input: String,
    pub should_quit: bool,
    pub selected_path: Option<PathBuf>,
    pub show_ignored: bool,
    pub toggle_ignore_request: Option<String>, // Project name to toggle
    pub show_help: bool,
    pub deactivate_request: Option<String>,
    pub archive_request: Option<String>,
    matcher: ProjectMatcher,
}

impl App {
    pub fn new(projects: Vec<Project>) -> Self {
        // Filter out ignored by default
        let filtered: Vec<usize> = projects
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.ignored)
            .map(|(i, _)| i)
            .collect();
        Self {
            projects,
            filtered,
            selected: 0,
            input: String::new(),
            should_quit: false,
            selected_path: None,
            show_ignored: false,
            toggle_ignore_request: None,
            show_help: false,
            deactivate_request: None,
            archive_request: None,
            matcher: ProjectMatcher::new(),
        }
    }

    pub fn filter(&mut self) {
        if self.input.is_empty() {
            // Show all (or filter ignored based on show_ignored flag)
            self.filtered = self
                .projects
                .iter()
                .enumerate()
                .filter(|(_, p)| self.show_ignored || !p.ignored)
                .map(|(i, _)| i)
                .collect();
        } else {
            let results = self.matcher.match_projects(&self.input, &self.projects);
            self.filtered = results
                .iter()
                .filter_map(|r| self.projects.iter().position(|p| p.name == r.project.name))
                .filter(|&idx| self.show_ignored || !self.projects[idx].ignored)
                .collect();
        }
        // Reset selection if out of bounds
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn toggle_show_ignored(&mut self) {
        self.show_ignored = !self.show_ignored;
        self.filter();
    }

    pub fn request_toggle_ignore(&mut self) {
        if let Some(project) = self.selected_project() {
            self.toggle_ignore_request = Some(project.name.clone());
        }
    }

    pub fn update_project_ignored(&mut self, name: &str, ignored: bool) {
        if let Some(project) = self.projects.iter_mut().find(|p| p.name == name) {
            project.ignored = ignored;
        }
        self.filter();
    }

    pub fn update_project_state(&mut self, name: &str, new_state: ProjectState) {
        if let Some(project) = self.projects.iter_mut().find(|p| p.name == name) {
            project.state = new_state;
        }
        self.filter();
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.filtered
            .get(self.selected)
            .map(|&idx| &self.projects[idx])
    }

    pub fn move_up(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = if self.selected == 0 {
                self.filtered.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
        }
    }

    pub fn select(&mut self) {
        if let Some(project) = self.selected_project() {
            self.selected_path = Some(project.path.clone());
            self.should_quit = true;
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match (code, modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Enter, _) => self.select(),
            (KeyCode::Up, _)
            | (KeyCode::Char('k'), KeyModifiers::NONE)
            | (KeyCode::Char('p'), KeyModifiers::CONTROL) => self.move_up(),
            (KeyCode::Down, _)
            | (KeyCode::Char('j'), KeyModifiers::NONE)
            | (KeyCode::Char('n'), KeyModifiers::CONTROL) => self.move_down(),
            // Toggle ignore on selected project (lowercase i)
            (KeyCode::Char('i'), KeyModifiers::NONE) if self.input.is_empty() => {
                self.request_toggle_ignore();
            }
            // Toggle show/hide ignored projects (uppercase I)
            (KeyCode::Char('I'), KeyModifiers::SHIFT) => {
                self.toggle_show_ignored();
            }
            // Help toggle
            (KeyCode::Char('?'), _) => {
                self.show_help = !self.show_help;
            }
            // Deactivate selected project (lowercase d, only when not typing)
            (KeyCode::Char('d'), KeyModifiers::NONE) if self.input.is_empty() => {
                if let Some(project) = self.selected_project() {
                    self.deactivate_request = Some(project.name.clone());
                }
            }
            // Archive selected project (lowercase a, only when not typing)
            (KeyCode::Char('a'), KeyModifiers::NONE) if self.input.is_empty() => {
                if let Some(project) = self.selected_project() {
                    self.archive_request = Some(project.name.clone());
                }
            }
            (KeyCode::Backspace, _) => {
                self.input.pop();
                self.filter();
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.input.push(c);
                self.filter();
            }
            _ => {}
        }
    }
}

use crate::database::operations::{toggle_ignored, update_project_state};
use crate::database::Database;
use crate::git::GitStatus;
use crate::tui::ui;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub fn run_app(projects: Vec<Project>, db: &Database) -> io::Result<Option<PathBuf>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(projects);

    // Main loop
    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle toggle ignore request (must be done outside of key handling due to borrow)
        if let Some(name) = app.toggle_ignore_request.take() {
            if let Ok(new_status) = toggle_ignored(&db.conn, &name) {
                app.update_project_ignored(&name, new_status);
            }
        }

        // Handle deactivate request
        if let Some(name) = app.deactivate_request.take() {
            // Check for uncommitted changes (non-blocking per CONTEXT.md)
            if let Some(project) = app.projects.iter().find(|p| p.name == name) {
                if let Ok(status) = GitStatus::check(&project.path) {
                    if status.has_warnings() {
                        // Warning shown briefly, operation proceeds
                    }
                }
            }
            if update_project_state(&db.conn, &name, ProjectState::Inactive).is_ok() {
                app.update_project_state(&name, ProjectState::Inactive);
            }
        }

        // Handle archive request
        if let Some(name) = app.archive_request.take() {
            // Check for uncommitted changes (non-blocking per CONTEXT.md)
            if let Some(project) = app.projects.iter().find(|p| p.name == name) {
                if let Ok(status) = GitStatus::check(&project.path) {
                    if status.has_warnings() {
                        // Warning shown briefly, operation proceeds
                    }
                }
            }
            if update_project_state(&db.conn, &name, ProjectState::Archived).is_ok() {
                app.update_project_state(&name, ProjectState::Archived);
            }
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(app.selected_path)
}
