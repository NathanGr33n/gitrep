//! Branch and Tag Browser Widget
//!
//! Displays branches and tags with navigation and operations.

use crate::git::{BranchDetail, TagInfo};
use crate::ui::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

/// Branch/Tag view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BranchViewTab {
    #[default]
    Branches,
    Tags,
}

/// Branch browser state
#[derive(Debug, Clone, Default)]
pub struct BranchBrowserState {
    /// Current tab
    pub tab: BranchViewTab,
    /// List of branches
    pub branches: Vec<BranchDetail>,
    /// List of tags
    pub tags: Vec<TagInfo>,
    /// Selected branch index
    pub selected_branch: usize,
    /// Selected tag index
    pub selected_tag: usize,
    /// Input mode for new branch/tag name
    pub input_mode: bool,
    /// Input buffer
    pub input_buffer: String,
    /// Operation type (for input mode)
    pub operation: BranchOperation,
}

/// Branch operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BranchOperation {
    #[default]
    None,
    CreateBranch,
    CreateTag,
}

impl BranchBrowserState {
    /// Create new branch browser state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update branches list
    pub fn update_branches(&mut self, branches: Vec<BranchDetail>) {
        self.branches = branches;
        if self.selected_branch >= self.branches.len() && !self.branches.is_empty() {
            self.selected_branch = self.branches.len() - 1;
        }
    }

    /// Update tags list
    pub fn update_tags(&mut self, tags: Vec<TagInfo>) {
        self.tags = tags;
        if self.selected_tag >= self.tags.len() && !self.tags.is_empty() {
            self.selected_tag = self.tags.len() - 1;
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            BranchViewTab::Branches => BranchViewTab::Tags,
            BranchViewTab::Tags => BranchViewTab::Branches,
        };
    }

    /// Select next item
    pub fn select_next(&mut self) {
        match self.tab {
            BranchViewTab::Branches => {
                if self.selected_branch < self.branches.len().saturating_sub(1) {
                    self.selected_branch += 1;
                }
            }
            BranchViewTab::Tags => {
                if self.selected_tag < self.tags.len().saturating_sub(1) {
                    self.selected_tag += 1;
                }
            }
        }
    }

    /// Select previous item
    pub fn select_previous(&mut self) {
        match self.tab {
            BranchViewTab::Branches => {
                self.selected_branch = self.selected_branch.saturating_sub(1);
            }
            BranchViewTab::Tags => {
                self.selected_tag = self.selected_tag.saturating_sub(1);
            }
        }
    }

    /// Get selected branch
    pub fn selected_branch_info(&self) -> Option<&BranchDetail> {
        self.branches.get(self.selected_branch)
    }

    /// Get selected tag
    pub fn selected_tag_info(&self) -> Option<&TagInfo> {
        self.tags.get(self.selected_tag)
    }

    /// Enter input mode for creating branch
    pub fn start_create_branch(&mut self) {
        self.input_mode = true;
        self.input_buffer.clear();
        self.operation = BranchOperation::CreateBranch;
    }

    /// Enter input mode for creating tag
    pub fn start_create_tag(&mut self) {
        self.input_mode = true;
        self.input_buffer.clear();
        self.operation = BranchOperation::CreateTag;
    }

    /// Exit input mode
    pub fn cancel_input(&mut self) {
        self.input_mode = false;
        self.input_buffer.clear();
        self.operation = BranchOperation::None;
    }

    /// Add character to input
    pub fn add_to_input(&mut self, c: char) {
        // Only allow valid branch/tag name characters
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.' {
            self.input_buffer.push(c);
        }
    }

    /// Remove last character from input
    pub fn backspace_input(&mut self) {
        self.input_buffer.pop();
    }
}

/// Render the branch browser
pub fn render_branch_browser(
    frame: &mut Frame,
    area: Rect,
    state: &BranchBrowserState,
    theme: &Theme,
) {
    // If in input mode, show input dialog
    if state.input_mode {
        render_input_dialog(frame, area, state, theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(area);

    // Render tabs
    let tab_titles = vec!["Branches", "Tags"];
    let selected_tab = match state.tab {
        BranchViewTab::Branches => 0,
        BranchViewTab::Tags => 1,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .select(selected_tab)
        .style(theme.normal)
        .highlight_style(theme.selected);

    frame.render_widget(tabs, chunks[0]);

    // Render content based on tab
    match state.tab {
        BranchViewTab::Branches => {
            render_branch_list(frame, chunks[1], state, theme);
        }
        BranchViewTab::Tags => {
            render_tag_list(frame, chunks[1], state, theme);
        }
    }
}

/// Render branch list
fn render_branch_list(
    frame: &mut Frame,
    area: Rect,
    state: &BranchBrowserState,
    theme: &Theme,
) {
    let items: Vec<ListItem> = state
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let is_selected = i == state.selected_branch;

            let prefix = if branch.is_head {
                "* "
            } else if branch.is_local {
                "  "
            } else {
                "  "
            };

            let name_style = if is_selected {
                theme.selected
            } else if branch.is_head {
                theme.branch
            } else if branch.is_local {
                theme.normal
            } else {
                theme.date
            };

            let mut spans = vec![
                Span::styled(prefix, name_style),
                Span::styled(&branch.name, name_style),
            ];

            // Add ahead/behind indicators for local branches with upstream
            if branch.is_local && (branch.ahead > 0 || branch.behind > 0) {
                let indicator = format!(" [↑{} ↓{}]", branch.ahead, branch.behind);
                spans.push(Span::styled(
                    indicator,
                    if is_selected { theme.selected } else { theme.commit_hash },
                ));
            }

            // Add latest commit info
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                &branch.latest_commit,
                if is_selected { theme.selected } else { theme.commit_hash },
            ));
            spans.push(Span::raw(" "));

            // Truncate message
            let max_msg_len = 30;
            let msg = if branch.latest_message.len() > max_msg_len {
                format!("{}…", &branch.latest_message[..max_msg_len - 1])
            } else {
                branch.latest_message.clone()
            };
            spans.push(Span::styled(
                msg,
                if is_selected { theme.selected } else { theme.normal },
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Branches ({}) ", state.branches.len()))
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .highlight_style(theme.selected);

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_branch));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render tag list
fn render_tag_list(
    frame: &mut Frame,
    area: Rect,
    state: &BranchBrowserState,
    theme: &Theme,
) {
    let items: Vec<ListItem> = state
        .tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let is_selected = i == state.selected_tag;

            let tag_type = if tag.is_lightweight { "○" } else { "●" };

            let name_style = if is_selected { theme.selected } else { theme.branch };

            let mut spans = vec![
                Span::styled(format!("{} ", tag_type), name_style),
                Span::styled(&tag.name, name_style),
                Span::raw(" → "),
                Span::styled(
                    &tag.target,
                    if is_selected { theme.selected } else { theme.commit_hash },
                ),
            ];

            // Add tagger for annotated tags
            if let Some(ref tagger) = tag.tagger {
                spans.push(Span::raw(" by "));
                spans.push(Span::styled(
                    tagger,
                    if is_selected { theme.selected } else { theme.author },
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Tags ({}) ", state.tags.len()))
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .highlight_style(theme.selected);

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_tag));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render input dialog
fn render_input_dialog(
    frame: &mut Frame,
    area: Rect,
    state: &BranchBrowserState,
    theme: &Theme,
) {
    let title = match state.operation {
        BranchOperation::CreateBranch => "Create Branch",
        BranchOperation::CreateTag => "Create Tag",
        BranchOperation::None => "",
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    // Instructions
    let instructions = Paragraph::new("Enter name (Enter to confirm, Esc to cancel)")
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        );
    frame.render_widget(instructions, chunks[0]);

    // Input field
    let input = Paragraph::new(format!("{}_", state.input_buffer))
        .block(
            Block::default()
                .title(" Name ")
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        );
    frame.render_widget(input, chunks[1]);
}
