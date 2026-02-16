//! Commit List Widget
//!
//! Displays a scrollable list of commits.

use crate::app::AppState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Render the commit list widget
pub fn render_commit_list(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let items: Vec<ListItem> = state
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let is_selected = i == state.selected_commit;

            let hash_style = if is_selected {
                theme.selected
            } else {
                theme.commit_hash
            };

            let msg_style = if is_selected {
                theme.selected
            } else {
                theme.normal
            };

            let author_style = if is_selected {
                theme.selected
            } else {
                theme.author
            };

            let date_style = if is_selected {
                theme.selected
            } else {
                theme.date
            };

            // Truncate summary to fit
            let max_summary_len = 40;
            let summary = if commit.summary.len() > max_summary_len {
                format!("{}…", &commit.summary[..max_summary_len - 1])
            } else {
                commit.summary.clone()
            };

            // Truncate author name
            let max_author_len = 12;
            let author = if commit.author_name.len() > max_author_len {
                format!("{}…", &commit.author_name[..max_author_len - 1])
            } else {
                commit.author_name.clone()
            };

            let line = Line::from(vec![
                Span::styled(&commit.short_id, hash_style),
                Span::raw(" "),
                Span::styled(summary, msg_style),
                Span::raw(" "),
                Span::styled(format!("<{}>", author), author_style),
                Span::raw(" "),
                Span::styled(commit.relative_time(), date_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Commits ")
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .highlight_style(theme.selected);

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_commit));

    frame.render_stateful_widget(list, area, &mut list_state);
}
