use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyModifiers};
use crate::database::models::Project;
use crate::navigation::ProjectMatcher;

pub struct App {
    pub projects: Vec<Project>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub input: String,
    pub should_quit: bool,
    pub selected_path: Option<PathBuf>,
    matcher: ProjectMatcher,
}

impl App {
    pub fn new(projects: Vec<Project>) -> Self {
        let filtered: Vec<usize> = (0..projects.len()).collect();
        Self {
            projects,
            filtered,
            selected: 0,
            input: String::new(),
            should_quit: false,
            selected_path: None,
            matcher: ProjectMatcher::new(),
        }
    }

    pub fn filter(&mut self) {
        if self.input.is_empty() {
            self.filtered = (0..self.projects.len()).collect();
        } else {
            let results = self.matcher.match_projects(&self.input, &self.projects);
            self.filtered = results
                .iter()
                .filter_map(|r| {
                    self.projects
                        .iter()
                        .position(|p| p.name == r.project.name)
                })
                .collect();
        }
        // Reset selection if out of bounds
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
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

// run_app will be implemented in Task 4
use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::tui::ui;

pub fn run_app(projects: Vec<Project>) -> io::Result<Option<PathBuf>> {
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
