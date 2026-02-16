//! Commit Information
//!
//! Data structures for representing commit information.

use chrono::{DateTime, TimeZone, Utc};
use git2::Commit;

/// Information about a Git commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Full commit hash
    pub id: String,
    /// Short commit hash (7 characters)
    pub short_id: String,
    /// Commit message (first line)
    pub summary: String,
    /// Full commit message
    pub message: String,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Parent commit IDs
    pub parents: Vec<String>,
}

impl CommitInfo {
    /// Create CommitInfo from a git2::Commit
    pub fn from_commit(commit: &Commit) -> Self {
        let id = commit.id().to_string();
        let short_id = id[..7.min(id.len())].to_string();

        let summary = commit
            .summary()
            .unwrap_or("(no message)")
            .to_string();

        let message = commit
            .message()
            .unwrap_or("(no message)")
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();

        let time = commit.time();
        let timestamp = Utc
            .timestamp_opt(time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        let parents = commit
            .parent_ids()
            .map(|oid| oid.to_string())
            .collect();

        Self {
            id,
            short_id,
            summary,
            message,
            author_name,
            author_email,
            timestamp,
            parents,
        }
    }

    /// Format the timestamp as a relative time string
    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.timestamp);

        if duration.num_days() > 365 {
            format!("{} years ago", duration.num_days() / 365)
        } else if duration.num_days() > 30 {
            format!("{} months ago", duration.num_days() / 30)
        } else if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    }

    /// Format the timestamp as a date string
    pub fn date_string(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }
}
