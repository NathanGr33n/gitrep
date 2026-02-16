//! Application Core
//!
//! Coordinates state management, async tasks, event loop, and communication
//! between the UI and Git layer.

mod config;
mod state;

pub use config::Config;
pub use state::AppState;

use crate::git::{Repository, StagingArea};
use crate::ui::{Event, Tui};
use crate::app::state::ViewMode;
use anyhow::{Context, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Main application struct
pub struct App {
    /// Application state
    pub state: AppState,
    /// Git repository
    pub repo: Repository,
    /// Application configuration
    pub config: Config,
    /// Whether the application should quit
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(path: PathBuf) -> Result<Self> {
        info!("Initializing gitrep for path: {:?}", path);

        let repo = Repository::open(&path).context("Failed to open Git repository")?;
        let config = Config::load().unwrap_or_default();
        let state = AppState::new(&repo)?;

        Ok(Self {
            state,
            repo,
            config,
            should_quit: false,
        })
    }

    /// Run the main application loop
    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        info!("Starting application main loop");

        while !self.should_quit {
            // Render the current state
            tui.draw(&self.state, &self.repo)?;

            // Handle events
            if let Some(event) = self.poll_event()? {
                self.handle_event(event)?;
            }
        }

        info!("Application shutting down");
        Ok(())
    }

    /// Poll for events with a timeout
    fn poll_event(&self) -> Result<Option<Event>> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Some(Event::Key(key))),
                CrosstermEvent::Mouse(mouse) => Ok(Some(Event::Mouse(mouse))),
                CrosstermEvent::Resize(w, h) => Ok(Some(Event::Resize(w, h))),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Handle an incoming event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(_mouse) => Ok(()), // Mouse support can be added later
            Event::Resize(_, _) => Ok(()),  // Handled automatically by ratatui
        }
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        debug!("Key event: {:?}", key);

        // Handle commit message input mode separately
        if self.state.view_mode == ViewMode::Commit {
            return self.handle_commit_input(key);
        }

        // Handle staging view separately
        if self.state.view_mode == ViewMode::Staging {
            return self.handle_staging_input(key);
        }

        // Global keybindings
        match (key.modifiers, key.code) {
            // Quit application
            (KeyModifiers::CONTROL, KeyCode::Char('c'))
            | (KeyModifiers::CONTROL, KeyCode::Char('q'))
            | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                self.should_quit = true;
            }

            // Navigation - Vim style
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
                self.state.select_next();
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
                self.state.select_previous();
            }
            (KeyModifiers::NONE, KeyCode::Char('g')) => {
                self.state.select_first();
            }
            (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                self.state.select_last();
            }

            // Page navigation
            (KeyModifiers::CONTROL, KeyCode::Char('d')) | (KeyModifiers::NONE, KeyCode::PageDown) => {
                self.state.page_down();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) | (KeyModifiers::NONE, KeyCode::PageUp) => {
                self.state.page_up();
            }

            // Tab navigation between panes
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.state.next_pane();
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.previous_pane();
            }

            // Enter to view commit details
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.state.toggle_detail_view();
            }

            // Escape to go back
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.state.go_back();
            }

            // Help
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                self.state.toggle_help();
            }

            // Search
            (KeyModifiers::NONE, KeyCode::Char('/')) => {
                self.state.start_search();
            }

            // Staging view
            (KeyModifiers::NONE, KeyCode::Char('s')) => {
                self.state.refresh_staging(&self.repo)?;
                self.state.enter_staging();
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle staging view input
    fn handle_staging_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match (key.modifiers, key.code) {
            // Quit
            (KeyModifiers::CONTROL, KeyCode::Char('c')) | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                self.should_quit = true;
            }

            // Navigation
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
                self.state.staging.select_next();
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
                self.state.staging.select_previous();
            }

            // Switch between staged/unstaged sections
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.state.staging.toggle_section();
            }

            // Stage/unstage file
            (KeyModifiers::NONE, KeyCode::Char(' ')) | (KeyModifiers::NONE, KeyCode::Enter) => {
                self.toggle_stage_file()?;
            }

            // Stage all
            (KeyModifiers::NONE, KeyCode::Char('a')) => {
                let staging_area = StagingArea::new(self.repo.inner());
                staging_area.stage_all()?;
                self.state.refresh_staging(&self.repo)?;
            }

            // Unstage all
            (KeyModifiers::NONE, KeyCode::Char('u')) => {
                let staging_area = StagingArea::new(self.repo.inner());
                staging_area.unstage_all()?;
                self.state.refresh_staging(&self.repo)?;
            }

            // Commit
            (KeyModifiers::NONE, KeyCode::Char('c')) => {
                self.state.enter_commit_mode();
            }

            // Go back
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.state.go_back();
            }

            // Help
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                self.state.toggle_help();
            }

            // Refresh
            (KeyModifiers::NONE, KeyCode::Char('r')) => {
                self.state.refresh_staging(&self.repo)?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle commit message input
    fn handle_commit_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.go_back();
            }
            KeyCode::Enter => {
                if !self.state.staging.commit_message.is_empty() {
                    self.create_commit()?;
                }
            }
            KeyCode::Backspace => {
                self.state.staging.backspace_message();
            }
            KeyCode::Char(c) => {
                self.state.staging.add_to_message(c);
            }
            _ => {}
        }

        Ok(())
    }

    /// Toggle staging for the selected file
    fn toggle_stage_file(&mut self) -> Result<()> {
        let staging_area = StagingArea::new(self.repo.inner());

        if let Some(file) = self.state.staging.selected_file() {
            let path = file.path.clone();
            if self.state.staging.in_staged_section {
                staging_area.unstage_file(&path)?;
            } else {
                staging_area.stage_file(&path)?;
            }
            self.state.refresh_staging(&self.repo)?;
        }

        Ok(())
    }

    /// Create a commit with the current staged changes
    fn create_commit(&mut self) -> Result<()> {
        let staging_area = StagingArea::new(self.repo.inner());
        let message = self.state.staging.commit_message.clone();

        match staging_area.commit(&message) {
            Ok(commit_id) => {
                info!("Created commit: {}", &commit_id[..7]);
                self.state.staging.exit_commit_mode();
                self.state.view_mode = ViewMode::Normal;

                // Refresh commits list
                self.state.commits = self.repo.get_commits(100)?;
                self.state.total_commits = self.state.commits.len();
            }
            Err(e) => {
                warn!("Failed to create commit: {}", e);
            }
        }

        Ok(())
    }
}
