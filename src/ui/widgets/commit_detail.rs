//! Commit Detail Widget
//!
//! Displays detailed information about a selected commit.

use crate::git::{CommitInfo, Repository};
use crate::ui::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the commit detail widget (side panel)
pub fn render_commit_detail(
    frame: &mut Frame,
    area: Rect,
    commit: &CommitInfo,
    _repo: &Repository,
    theme: &Theme,
) {
    let content = build_commit_detail_text(commit, theme);

    let detail = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Commit Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, area);
}

/// Render the full-screen commit detail view
pub fn render_commit_detail_full(
    frame: &mut Frame,
    area: Rect,
    commit: &CommitInfo,
    repo: &Repository,
    theme: &Theme,
) {
    // Split into metadata and diff sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(10)])
        .split(area);

    // Render metadata
    let content = build_commit_detail_text(commit, theme);
    let metadata = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Commit Detail ")
                .borders(Borders::ALL)
                .border_style(theme.border_active),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(metadata, chunks[0]);

    // Render diff
    render_diff(frame, chunks[1], commit, repo, theme);
}

/// Build the commit detail text
fn build_commit_detail_text<'a>(commit: &CommitInfo, theme: &Theme) -> Text<'a> {
    let lines = vec![
        Line::from(vec![
            Span::styled("Commit:  ", theme.help_key),
            Span::styled(commit.id.clone(), theme.commit_hash),
        ]),
        Line::from(vec![
            Span::styled("Author:  ", theme.help_key),
            Span::styled(commit.author_name.clone(), theme.author),
            Span::raw(" <"),
            Span::styled(commit.author_email.clone(), theme.normal),
            Span::raw(">"),
        ]),
        Line::from(vec![
            Span::styled("Date:    ", theme.help_key),
            Span::styled(commit.date_string(), theme.date),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(commit.message.clone(), theme.normal)]),
    ];

    Text::from(lines)
}

/// Render the diff section
fn render_diff(
    frame: &mut Frame,
    area: Rect,
    commit: &CommitInfo,
    repo: &Repository,
    theme: &Theme,
) {
    let diff_text = repo
        .get_commit_diff(&commit.id)
        .unwrap_or_else(|_| "(unable to load diff)".to_string());

    let lines: Vec<Line> = diff_text
        .lines()
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                theme.diff_add
            } else if line.starts_with('-') && !line.starts_with("---") {
                theme.diff_remove
            } else if line.starts_with("@@") {
                theme.diff_hunk
            } else {
                theme.normal
            };

            Line::from(Span::styled(line.to_string(), style))
        })
        .collect();

    let diff = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Diff ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(diff, area);
}
