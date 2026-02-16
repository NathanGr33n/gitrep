//! Header Widget
//!
//! Displays the repository name, current branch, and other status info.

use crate::app::AppState;
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the header widget
pub fn render_header(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title_spans = vec![
        Span::styled(" gitrep ", theme.header),
        Span::raw("│ "),
        Span::styled(&state.repo_name, theme.normal),
        Span::raw(" │ "),
        Span::styled("⎇ ", theme.branch),
        Span::styled(&state.current_branch, theme.branch),
        Span::raw(" │ "),
        Span::styled(format!("{} commits", state.total_commits), theme.date),
    ];

    let header = Paragraph::new(Line::from(title_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        );

    frame.render_widget(header, area);
}
