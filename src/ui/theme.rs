//! Theme Configuration
//!
//! Defines colors and styles for the TUI.

use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Normal text style
    pub normal: Style,
    /// Highlighted/selected item style
    pub selected: Style,
    /// Header style
    pub header: Style,
    /// Border style
    pub border: Style,
    /// Active border style (focused pane)
    pub border_active: Style,
    /// Commit hash style
    pub commit_hash: Style,
    /// Author name style
    pub author: Style,
    /// Date/time style
    pub date: Style,
    /// Branch name style
    pub branch: Style,
    /// Added line style (diff)
    pub diff_add: Style,
    /// Removed line style (diff)
    pub diff_remove: Style,
    /// Hunk header style (diff)
    pub diff_hunk: Style,
    /// Help text style
    pub help_key: Style,
    /// Help description style
    pub help_desc: Style,
    /// Status bar style
    pub status_bar: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            selected: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            border_active: Style::default().fg(Color::Cyan),
            commit_hash: Style::default().fg(Color::Yellow),
            author: Style::default().fg(Color::Green),
            date: Style::default().fg(Color::Blue),
            branch: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            diff_add: Style::default().fg(Color::Green),
            diff_remove: Style::default().fg(Color::Red),
            diff_hunk: Style::default().fg(Color::Cyan),
            help_key: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(Color::White),
            status_bar: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan),
        }
    }
}

impl Theme {
    /// Create a dark theme variant
    pub fn dark() -> Self {
        Self::default()
    }

    /// Create a light theme variant
    pub fn light() -> Self {
        Self {
            normal: Style::default().fg(Color::Black),
            selected: Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Gray),
            border_active: Style::default().fg(Color::Blue),
            commit_hash: Style::default().fg(Color::Rgb(180, 100, 0)),
            author: Style::default().fg(Color::Rgb(0, 128, 0)),
            date: Style::default().fg(Color::Rgb(0, 0, 180)),
            branch: Style::default()
                .fg(Color::Rgb(128, 0, 128))
                .add_modifier(Modifier::BOLD),
            diff_add: Style::default().fg(Color::Rgb(0, 128, 0)),
            diff_remove: Style::default().fg(Color::Rgb(180, 0, 0)),
            diff_hunk: Style::default().fg(Color::Blue),
            help_key: Style::default()
                .fg(Color::Rgb(180, 100, 0))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(Color::Black),
            status_bar: Style::default()
                .fg(Color::White)
                .bg(Color::Blue),
        }
    }
}
