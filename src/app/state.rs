//! Application State Management
//!
//! Manages the state of the TUI application including selected items,
//! current view, and navigation history.

use crate::git::{CommitInfo, Repository, StagingArea};
use crate::ui::widgets::StagingState;
use anyhow::Result;

/// The currently active pane in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePane {
    #[default]
    CommitList,
    CommitDetail,
    BranchList,
    Help,
}

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Normal,
    Detail,
    Search,
    Help,
    Staging,
    Commit,
}

/// Application state
#[derive(Debug)]
pub struct AppState {
    /// Currently active pane
    pub active_pane: ActivePane,
    /// Current view mode
    pub view_mode: ViewMode,
    /// List of commits (lazily loaded)
    pub commits: Vec<CommitInfo>,
    /// Currently selected commit index
    pub selected_commit: usize,
    /// Scroll offset for the commit list
    pub scroll_offset: usize,
    /// Number of visible rows (updated on resize)
    pub visible_rows: usize,
    /// Current search query
    pub search_query: String,
    /// Repository name
    pub repo_name: String,
    /// Current branch name
    pub current_branch: String,
    /// Total commit count (for display)
    pub total_commits: usize,
    /// Staging area state
    pub staging: StagingState,
}

impl AppState {
    /// Create new application state
    pub fn new(repo: &Repository) -> Result<Self> {
        let commits = repo.get_commits(100)?; // Load initial batch
        let total_commits = commits.len();
        let repo_name = repo.name().to_string();
        let current_branch = repo.current_branch().unwrap_or_else(|| "HEAD".to_string());

        Ok(Self {
            active_pane: ActivePane::CommitList,
            view_mode: ViewMode::Normal,
            commits,
            selected_commit: 0,
            scroll_offset: 0,
            visible_rows: 20, // Default, will be updated
            search_query: String::new(),
            repo_name,
            current_branch,
            total_commits,
            staging: StagingState::new(),
        })
    }

    /// Refresh staging area status
    pub fn refresh_staging(&mut self, repo: &Repository) -> Result<()> {
        let staging_area = StagingArea::new(repo.inner());
        let staged = staging_area.get_staged()?;
        let unstaged = staging_area.get_unstaged()?;
        self.staging.update_files(staged, unstaged);
        Ok(())
    }

    /// Enter staging view
    pub fn enter_staging(&mut self) {
        self.view_mode = ViewMode::Staging;
    }

    /// Enter commit mode
    pub fn enter_commit_mode(&mut self) {
        if self.staging.has_staged_changes() {
            self.view_mode = ViewMode::Commit;
            self.staging.enter_commit_mode();
        }
    }

    /// Select the next item in the current list
    pub fn select_next(&mut self) {
        if self.commits.is_empty() {
            return;
        }

        if self.selected_commit < self.commits.len() - 1 {
            self.selected_commit += 1;

            // Scroll down if needed
            if self.selected_commit >= self.scroll_offset + self.visible_rows {
                self.scroll_offset = self.selected_commit - self.visible_rows + 1;
            }
        }
    }

    /// Select the previous item in the current list
    pub fn select_previous(&mut self) {
        if self.selected_commit > 0 {
            self.selected_commit -= 1;

            // Scroll up if needed
            if self.selected_commit < self.scroll_offset {
                self.scroll_offset = self.selected_commit;
            }
        }
    }

    /// Select the first item
    pub fn select_first(&mut self) {
        self.selected_commit = 0;
        self.scroll_offset = 0;
    }

    /// Select the last item
    pub fn select_last(&mut self) {
        if !self.commits.is_empty() {
            self.selected_commit = self.commits.len() - 1;
            if self.commits.len() > self.visible_rows {
                self.scroll_offset = self.commits.len() - self.visible_rows;
            }
        }
    }

    /// Page down
    pub fn page_down(&mut self) {
        let page_size = self.visible_rows.saturating_sub(2);
        let new_selected = (self.selected_commit + page_size).min(self.commits.len().saturating_sub(1));
        self.selected_commit = new_selected;

        if self.selected_commit >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_commit.saturating_sub(self.visible_rows - 1);
        }
    }

    /// Page up
    pub fn page_up(&mut self) {
        let page_size = self.visible_rows.saturating_sub(2);
        self.selected_commit = self.selected_commit.saturating_sub(page_size);

        if self.selected_commit < self.scroll_offset {
            self.scroll_offset = self.selected_commit;
        }
    }

    /// Switch to the next pane
    pub fn next_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::CommitList => ActivePane::CommitDetail,
            ActivePane::CommitDetail => ActivePane::BranchList,
            ActivePane::BranchList => ActivePane::CommitList,
            ActivePane::Help => ActivePane::CommitList,
        };
    }

    /// Switch to the previous pane
    pub fn previous_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::CommitList => ActivePane::BranchList,
            ActivePane::CommitDetail => ActivePane::CommitList,
            ActivePane::BranchList => ActivePane::CommitDetail,
            ActivePane::Help => ActivePane::CommitList,
        };
    }

    /// Toggle detail view
    pub fn toggle_detail_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Normal => ViewMode::Detail,
            ViewMode::Detail => ViewMode::Normal,
            _ => self.view_mode,
        };
    }

    /// Go back from current view
    pub fn go_back(&mut self) {
        match self.view_mode {
            ViewMode::Detail | ViewMode::Search | ViewMode::Help | ViewMode::Staging => {
                self.view_mode = ViewMode::Normal;
            }
            ViewMode::Commit => {
                self.staging.exit_commit_mode();
                self.view_mode = ViewMode::Staging;
            }
            ViewMode::Normal => {}
        }
    }

    /// Toggle help view
    pub fn toggle_help(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Help => ViewMode::Normal,
            _ => ViewMode::Help,
        };
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.view_mode = ViewMode::Search;
        self.search_query.clear();
    }

    /// Get the currently selected commit
    pub fn selected_commit_info(&self) -> Option<&CommitInfo> {
        self.commits.get(self.selected_commit)
    }

    /// Update visible rows (called on resize)
    pub fn set_visible_rows(&mut self, rows: usize) {
        self.visible_rows = rows.saturating_sub(4); // Account for borders/header
    }
}
