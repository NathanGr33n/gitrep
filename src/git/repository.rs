//! Repository Abstraction
//!
//! Wraps git2::Repository with application-specific functionality.

use super::CommitInfo;
use anyhow::{Context, Result};
use git2::{Repository as Git2Repository, Sort};
use std::path::Path;
use tracing::{debug, info};

/// Wrapper around git2::Repository providing high-level operations
pub struct Repository {
    inner: Git2Repository,
}

impl Repository {
    /// Open a repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        info!("Opening repository at: {:?}", path);

        let repo = Git2Repository::discover(path).context("Not a Git repository (or any parent)")?;

        debug!("Repository found at: {:?}", repo.path());

        Ok(Self { inner: repo })
    }

    /// Get the repository name (directory name)
    pub fn name(&self) -> &str {
        self.inner
            .workdir()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Option<String> {
        let head = self.inner.head().ok()?;

        if head.is_branch() {
            head.shorthand().map(|s| s.to_string())
        } else {
            // Detached HEAD - return short commit id
            head.target().map(|oid| format!("{:.7}", oid))
        }
    }

    /// Get a list of commits with optional limit
    pub fn get_commits(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.inner.revwalk()?;

        // Start from HEAD
        revwalk.push_head()?;

        // Sort by time, newest first
        revwalk.set_sorting(Sort::TIME)?;

        let mut commits = Vec::with_capacity(limit);

        for (i, oid_result) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid_result?;
            let commit = self.inner.find_commit(oid)?;
            commits.push(CommitInfo::from_commit(&commit));
        }

        debug!("Loaded {} commits", commits.len());
        Ok(commits)
    }

    /// Get branches in the repository
    pub fn get_branches(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();

        for branch_result in self.inner.branches(None)? {
            let (branch, branch_type) = branch_result?;

            let name = branch.name()?.unwrap_or("unknown").to_string();

            let is_head = branch.is_head();

            let upstream_name = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

            branches.push(BranchInfo {
                name,
                is_local: branch_type == git2::BranchType::Local,
                is_head,
                upstream: upstream_name,
            });
        }

        Ok(branches)
    }

    /// Get a specific commit by its ID
    pub fn get_commit(&self, id: &str) -> Result<Option<CommitInfo>> {
        let oid = git2::Oid::from_str(id)?;

        match self.inner.find_commit(oid) {
            Ok(commit) => Ok(Some(CommitInfo::from_commit(&commit))),
            Err(_) => Ok(None),
        }
    }

    /// Get the diff for a specific commit
    pub fn get_commit_diff(&self, commit_id: &str) -> Result<String> {
        let oid = git2::Oid::from_str(commit_id)?;
        let commit = self.inner.find_commit(oid)?;

        let tree = commit.tree()?;

        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = self.inner.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut diff_text = String::new();

        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                _ => "",
            };

            if let Ok(content) = std::str::from_utf8(line.content()) {
                diff_text.push_str(prefix);
                diff_text.push_str(content);
            }

            true
        })?;

        Ok(diff_text)
    }

    /// Get files changed in a commit
    pub fn get_commit_files(&self, commit_id: &str) -> Result<Vec<FileChange>> {
        let oid = git2::Oid::from_str(commit_id)?;
        let commit = self.inner.find_commit(oid)?;

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = self.inner.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut files = Vec::new();

        for delta in diff.deltas() {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let status = match delta.status() {
                git2::Delta::Added => FileStatus::Added,
                git2::Delta::Deleted => FileStatus::Deleted,
                git2::Delta::Modified => FileStatus::Modified,
                git2::Delta::Renamed => FileStatus::Renamed,
                git2::Delta::Copied => FileStatus::Copied,
                _ => FileStatus::Modified,
            };

            files.push(FileChange { path, status });
        }

        Ok(files)
    }

    /// Check if the repository has uncommitted changes
    pub fn has_changes(&self) -> Result<bool> {
        let statuses = self.inner.statuses(None)?;
        Ok(!statuses.is_empty())
    }

    /// Get the inner git2 repository reference (for advanced operations)
    pub fn inner(&self) -> &Git2Repository {
        &self.inner
    }
}

/// Information about a branch
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_local: bool,
    pub is_head: bool,
    pub upstream: Option<String>,
}

/// Status of a file change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
}

impl FileStatus {
    /// Get a single character representation
    pub fn as_char(&self) -> char {
        match self {
            FileStatus::Added => 'A',
            FileStatus::Deleted => 'D',
            FileStatus::Modified => 'M',
            FileStatus::Renamed => 'R',
            FileStatus::Copied => 'C',
        }
    }
}

/// Information about a file change
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
}
