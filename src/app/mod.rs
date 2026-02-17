//! Application Core
//!
//! Coordinates state management, async tasks, event loop, and communication
//! between the UI and Git layer.

mod config;
pub(crate) mod state;

pub use config::Config;
pub use state::AppState;

use crate::git::{BranchManager, Repository, StagingArea};
use crate::ui::{Event, Tui};
use crate::ui::widgets::{BranchOperation, BranchViewTab};
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

        // Handle branches view separately
        if self.state.view_mode == ViewMode::Branches {
            return self.handle_branches_input(key);
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

            // Branches view
            (KeyModifiers::NONE, KeyCode::Char('b')) => {
                self.state.refresh_branches(&self.repo)?;
                self.state.enter_branches();
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

    /// Handle branches view input
    fn handle_branches_input(&mut self, key: event::KeyEvent) -> Result<()> {
        // Handle input mode for creating branch/tag
        if self.state.branch_browser.input_mode {
            return self.handle_branch_name_input(key);
        }

        match (key.modifiers, key.code) {
            // Quit
            (KeyModifiers::CONTROL, KeyCode::Char('c')) | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                self.should_quit = true;
            }

            // Navigation
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
                self.state.branch_browser.select_next();
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
                self.state.branch_browser.select_previous();
            }

            // Switch tabs
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.state.branch_browser.next_tab();
            }

            // Checkout branch
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.checkout_selected_branch()?;
            }

            // Create branch
            (KeyModifiers::NONE, KeyCode::Char('n')) => {
                if self.state.branch_browser.tab == BranchViewTab::Branches {
                    self.state.branch_browser.start_create_branch();
                } else {
                    self.state.branch_browser.start_create_tag();
                }
            }

            // Delete branch/tag
            (KeyModifiers::NONE, KeyCode::Char('d')) => {
                self.delete_selected_branch_or_tag()?;
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
                self.state.refresh_branches(&self.repo)?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle branch/tag name input
    fn handle_branch_name_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.branch_browser.cancel_input();
            }
            KeyCode::Enter => {
                if !self.state.branch_browser.input_buffer.is_empty() {
                    self.create_branch_or_tag()?;
                }
            }
            KeyCode::Backspace => {
                self.state.branch_browser.backspace_input();
            }
            KeyCode::Char(c) => {
                self.state.branch_browser.add_to_input(c);
            }
            _ => {}
        }

        Ok(())
    }

    /// Checkout the selected branch
    fn checkout_selected_branch(&mut self) -> Result<()> {
        if self.state.branch_browser.tab != BranchViewTab::Branches {
            return Ok(());
        }

        if let Some(branch) = self.state.branch_browser.selected_branch_info() {
            if branch.is_local && !branch.is_head {
                let branch_mgr = BranchManager::new(self.repo.inner());
                match branch_mgr.checkout_branch(&branch.name) {
                    Ok(()) => {
                        info!("Checked out branch: {}", branch.name);
                        self.state.refresh_branches(&self.repo)?;
                        // Refresh commits for new branch
                        self.state.commits = self.repo.get_commits(100)?;
                        self.state.total_commits = self.state.commits.len();
                        self.state.selected_commit = 0;
                    }
                    Err(e) => {
                        warn!("Failed to checkout branch: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a new branch or tag
    fn create_branch_or_tag(&mut self) -> Result<()> {
        let branch_mgr = BranchManager::new(self.repo.inner());
        let name = self.state.branch_browser.input_buffer.clone();

        match self.state.branch_browser.operation {
            BranchOperation::CreateBranch => {
                match branch_mgr.create_branch(&name, true) {
                    Ok(()) => {
                        info!("Created branch: {}", name);
                    }
                    Err(e) => {
                        warn!("Failed to create branch: {}", e);
                    }
                }
            }
            BranchOperation::CreateTag => {
                match branch_mgr.create_tag(&name, None) {
                    Ok(()) => {
                        info!("Created tag: {}", name);
                    }
                    Err(e) => {
                        warn!("Failed to create tag: {}", e);
                    }
                }
            }
            BranchOperation::None => {}
        }

        self.state.branch_browser.cancel_input();
        self.state.refresh_branches(&self.repo)?;

        Ok(())
    }

    /// Delete the selected branch or tag
    fn delete_selected_branch_or_tag(&mut self) -> Result<()> {
        let branch_mgr = BranchManager::new(self.repo.inner());

        match self.state.branch_browser.tab {
            BranchViewTab::Branches => {
                if let Some(branch) = self.state.branch_browser.selected_branch_info() {
                    if branch.is_local && !branch.is_head {
                        match branch_mgr.delete_branch(&branch.name, false) {
                            Ok(()) => {
                                info!("Deleted branch: {}", branch.name);
                            }
                            Err(e) => {
                                warn!("Failed to delete branch: {}", e);
                            }
                        }
                    }
                }
            }
            BranchViewTab::Tags => {
                if let Some(tag) = self.state.branch_browser.selected_tag_info() {
                    match branch_mgr.delete_tag(&tag.name) {
                        Ok(()) => {
                            info!("Deleted tag: {}", tag.name);
                        }
                        Err(e) => {
                            warn!("Failed to delete tag: {}", e);
                        }
                    }
                }
            }
        }

        self.state.refresh_branches(&self.repo)?;

        Ok(())
    }
}
