//! Status Bar Widget
//!
//! Displays keybindings and status information at the bottom.

use crate::app::state::{AppState, ViewMode};
use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the status bar widget
pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mode_text = match state.view_mode {
        ViewMode::Normal => "NORMAL",
        ViewMode::Detail => "DETAIL",
        ViewMode::Search => "SEARCH",
        ViewMode::Help => "HELP",
        ViewMode::Staging => "STAGING",
        ViewMode::Commit => "COMMIT",
    };

    let help_text = match state.view_mode {
        ViewMode::Normal => "j/k:nav  Enter:detail  s:staging  ?:help  q:quit",
        ViewMode::Detail => "j/k:nav  Esc:back  ?:help  q:quit",
        ViewMode::Search => "Enter:search  Esc:cancel",
        ViewMode::Help => "Esc:back  q:quit",
        ViewMode::Staging => "j/k:nav  Space:stage  a:all  c:commit  Esc:back",
        ViewMode::Commit => "Enter:commit  Esc:cancel",
    };

    let position = format!(
        " {}/{} ",
        state.selected_commit + 1,
        state.commits.len()
    );

    let status_line = Line::from(vec![
        Span::styled(format!(" {} ", mode_text), theme.status_bar),
        Span::raw(" "),
        Span::styled(help_text, theme.help_desc),
        Span::raw(" "),
        Span::styled(position, theme.status_bar),
    ]);

    let status_bar = Paragraph::new(status_line);
    frame.render_widget(status_bar, area);
}
