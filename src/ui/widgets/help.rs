//! Help Widget
//!
//! Displays keyboard shortcuts and help information.

use crate::ui::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the help widget
pub fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_items = vec![
        ("Navigation", vec![
            ("j / ↓", "Move down"),
            ("k / ↑", "Move up"),
            ("g", "Go to first commit"),
            ("G", "Go to last commit"),
            ("Ctrl+d / PgDn", "Page down"),
            ("Ctrl+u / PgUp", "Page up"),
        ]),
        ("Actions", vec![
            ("Enter", "Toggle detail view"),
            ("Tab", "Next pane"),
            ("Shift+Tab", "Previous pane"),
            ("/", "Search"),
            ("?", "Toggle help"),
        ]),
        ("General", vec![
            ("q", "Quit"),
            ("Ctrl+c", "Force quit"),
            ("Esc", "Go back / Cancel"),
        ]),
    ];

    let mut lines = vec![
        Line::from(Span::styled("GitRep - Keyboard Shortcuts", theme.header)),
        Line::from(""),
    ];

    for (section, items) in help_items {
        lines.push(Line::from(Span::styled(section, theme.branch)));
        lines.push(Line::from(""));

        for (key, desc) in items {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:16}", key), theme.help_key),
                Span::styled(desc, theme.help_desc),
            ]));
        }

        lines.push(Line::from(""));
    }

    let help = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, area);
}
