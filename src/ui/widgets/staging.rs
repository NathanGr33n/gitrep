//! Staging Area Widget
//!
//! Displays working tree changes with staged and unstaged sections.

use crate::git::{WorkingTreeFile, WorkingTreeStatus};
use crate::ui::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Staging view state
#[derive(Debug, Clone, Default)]
pub struct StagingState {
    /// Staged files
    pub staged_files: Vec<WorkingTreeFile>,
    /// Unstaged files
    pub unstaged_files: Vec<WorkingTreeFile>,
    /// Currently selected section (true = staged, false = unstaged)
    pub in_staged_section: bool,
    /// Selected index in staged section
    pub staged_selected: usize,
    /// Selected index in unstaged section
    pub unstaged_selected: usize,
    /// Whether we're in commit message input mode
    pub commit_mode: bool,
    /// Current commit message
    pub commit_message: String,
}

impl StagingState {
    /// Create new staging state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the file lists
    pub fn update_files(&mut self, staged: Vec<WorkingTreeFile>, unstaged: Vec<WorkingTreeFile>) {
        self.staged_files = staged;
        self.unstaged_files = unstaged;

        // Adjust selection if needed
        if self.staged_selected >= self.staged_files.len() && !self.staged_files.is_empty() {
            self.staged_selected = self.staged_files.len() - 1;
        }
        if self.unstaged_selected >= self.unstaged_files.len() && !self.unstaged_files.is_empty() {
            self.unstaged_selected = self.unstaged_files.len() - 1;
        }
    }

    /// Get the currently selected file
    pub fn selected_file(&self) -> Option<&WorkingTreeFile> {
        if self.in_staged_section {
            self.staged_files.get(self.staged_selected)
        } else {
            self.unstaged_files.get(self.unstaged_selected)
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.in_staged_section {
            if self.staged_selected < self.staged_files.len().saturating_sub(1) {
                self.staged_selected += 1;
            }
        } else if self.unstaged_selected < self.unstaged_files.len().saturating_sub(1) {
            self.unstaged_selected += 1;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.in_staged_section {
            self.staged_selected = self.staged_selected.saturating_sub(1);
        } else {
            self.unstaged_selected = self.unstaged_selected.saturating_sub(1);
        }
    }

    /// Switch between staged and unstaged sections
    pub fn toggle_section(&mut self) {
        self.in_staged_section = !self.in_staged_section;
    }

    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> bool {
        !self.staged_files.is_empty()
    }

    /// Enter commit mode
    pub fn enter_commit_mode(&mut self) {
        self.commit_mode = true;
        self.commit_message.clear();
    }

    /// Exit commit mode
    pub fn exit_commit_mode(&mut self) {
        self.commit_mode = false;
        self.commit_message.clear();
    }

    /// Add character to commit message
    pub fn add_to_message(&mut self, c: char) {
        self.commit_message.push(c);
    }

    /// Remove last character from commit message
    pub fn backspace_message(&mut self) {
        self.commit_message.pop();
    }
}

/// Render the staging area view
pub fn render_staging(
    frame: &mut Frame,
    area: Rect,
    state: &StagingState,
    theme: &Theme,
) {
    // Split into staged and unstaged sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render staged files
    render_file_list(
        frame,
        chunks[0],
        "Staged Changes",
        &state.staged_files,
        state.staged_selected,
        state.in_staged_section,
        theme,
    );

    // Render unstaged files
    render_file_list(
        frame,
        chunks[1],
        "Unstaged Changes",
        &state.unstaged_files,
        state.unstaged_selected,
        !state.in_staged_section,
        theme,
    );
}

/// Render a list of files
fn render_file_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    files: &[WorkingTreeFile],
    selected: usize,
    is_focused: bool,
    theme: &Theme,
) {
    let items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = i == selected && is_focused;

            let status_style = get_status_style(&file.status, theme);
            let path_style = if is_selected { theme.selected } else { theme.normal };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", file.status.as_char()),
                    if is_selected { theme.selected } else { status_style },
                ),
                Span::styled(&file.path, path_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let border_style = if is_focused {
        theme.border_active
    } else {
        theme.border
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {} ({}) ", title, files.len()))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(theme.selected);

    let mut list_state = ListState::default();
    if is_focused && !files.is_empty() {
        list_state.select(Some(selected));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render the commit message input
pub fn render_commit_input(
    frame: &mut Frame,
    area: Rect,
    message: &str,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Title
    let title = Paragraph::new("Enter commit message (Enter to commit, Esc to cancel)")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        );
    frame.render_widget(title, chunks[0]);

    // Message input
    let input = Paragraph::new(format!("{}_", message))
        .block(
            Block::default()
                .title(" Message ")
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        );
    frame.render_widget(input, chunks[1]);
}

/// Get the style for a file status
fn get_status_style(status: &WorkingTreeStatus, theme: &Theme) -> ratatui::style::Style {
    match status {
        WorkingTreeStatus::StagedNew | WorkingTreeStatus::Untracked => theme.diff_add,
        WorkingTreeStatus::StagedDeleted | WorkingTreeStatus::Deleted => theme.diff_remove,
        WorkingTreeStatus::StagedModified | WorkingTreeStatus::Modified => theme.commit_hash,
        WorkingTreeStatus::Conflicted => theme.diff_remove,
        WorkingTreeStatus::Renamed => theme.branch,
    }
}
