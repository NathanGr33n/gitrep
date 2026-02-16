//! TUI Handler
//!
//! Manages terminal initialization, cleanup, and rendering.

use super::theme::Theme;
use super::widgets;
use crate::app::state::{AppState, ViewMode};
use crate::git::Repository;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};
use std::io::{self, Stdout};

/// TUI wrapper struct
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    theme: Theme,
}

impl Tui {
    /// Create a new TUI instance
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            theme: Theme::default(),
        })
    }

    /// Enter TUI mode (raw mode + alternate screen)
    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Exit TUI mode
    pub fn exit(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Draw the UI
    pub fn draw(&mut self, state: &AppState, repo: &Repository) -> Result<()> {
        let theme = &self.theme;

        self.terminal.draw(|frame| {
            let size = frame.area();

            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Main content
                    Constraint::Length(1),  // Status bar
                ])
                .split(size);

            // Render header
            widgets::render_header(frame, chunks[0], state, theme);

            // Render main content based on view mode
            match state.view_mode {
                ViewMode::Normal => {
                    render_normal_view(frame, chunks[1], state, repo, theme);
                }
                ViewMode::Detail => {
                    render_detail_view(frame, chunks[1], state, repo, theme);
                }
                ViewMode::Help => {
                    widgets::render_help(frame, chunks[1], theme);
                }
                ViewMode::Search => {
                    render_normal_view(frame, chunks[1], state, repo, theme);
                    // TODO: Add search overlay
                }
                ViewMode::Staging => {
                    widgets::render_staging(frame, chunks[1], &state.staging, theme);
                }
                ViewMode::Commit => {
                    widgets::render_commit_input(frame, chunks[1], &state.staging.commit_message, theme);
                }
            }

            // Render status bar
            widgets::render_status_bar(frame, chunks[2], state, theme);
        })?;

        Ok(())
    }
}

/// Render the normal view with commit list and detail panels
fn render_normal_view(
    frame: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    repo: &Repository,
    theme: &Theme,
) {
    // Split into commit list and detail pane
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render commit list
    widgets::render_commit_list(frame, chunks[0], state, theme);

    // Render commit detail (if a commit is selected)
    if let Some(commit) = state.selected_commit_info() {
        widgets::render_commit_detail(frame, chunks[1], commit, repo, theme);
    }
}

/// Render the detail view (full-screen commit detail)
fn render_detail_view(
    frame: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    repo: &Repository,
    theme: &Theme,
) {
    if let Some(commit) = state.selected_commit_info() {
        widgets::render_commit_detail_full(frame, area, commit, repo, theme);
    }
}
