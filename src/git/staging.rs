//! Staging Area Management
//!
//! Handles staging, unstaging, and working tree status operations.

use anyhow::Result;
use git2::{Repository as Git2Repository, Status, StatusOptions};
use std::path::Path;

/// Status of a file in the working tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingTreeStatus {
    /// File is new and untracked
    Untracked,
    /// File is staged for addition
    StagedNew,
    /// File is staged with modifications
    StagedModified,
    /// File is staged for deletion
    StagedDeleted,
    /// File has unstaged modifications
    Modified,
    /// File has been deleted but not staged
    Deleted,
    /// File has both staged and unstaged changes
    Conflicted,
    /// File is renamed
    Renamed,
}

impl WorkingTreeStatus {
    /// Get a single character representation
    pub fn as_char(&self) -> char {
        match self {
            WorkingTreeStatus::Untracked => '?',
            WorkingTreeStatus::StagedNew => 'A',
            WorkingTreeStatus::StagedModified => 'M',
            WorkingTreeStatus::StagedDeleted => 'D',
            WorkingTreeStatus::Modified => 'm',
            WorkingTreeStatus::Deleted => 'd',
            WorkingTreeStatus::Conflicted => 'C',
            WorkingTreeStatus::Renamed => 'R',
        }
    }

    /// Get a description of the status
    pub fn description(&self) -> &'static str {
        match self {
            WorkingTreeStatus::Untracked => "untracked",
            WorkingTreeStatus::StagedNew => "staged (new)",
            WorkingTreeStatus::StagedModified => "staged (modified)",
            WorkingTreeStatus::StagedDeleted => "staged (deleted)",
            WorkingTreeStatus::Modified => "modified",
            WorkingTreeStatus::Deleted => "deleted",
            WorkingTreeStatus::Conflicted => "conflicted",
            WorkingTreeStatus::Renamed => "renamed",
        }
    }

    /// Check if this status represents a staged file
    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            WorkingTreeStatus::StagedNew
                | WorkingTreeStatus::StagedModified
                | WorkingTreeStatus::StagedDeleted
        )
    }
}

/// Information about a file in the working tree
#[derive(Debug, Clone)]
pub struct WorkingTreeFile {
    /// File path relative to repository root
    pub path: String,
    /// Current status
    pub status: WorkingTreeStatus,
    /// Old path (for renames)
    pub old_path: Option<String>,
}

/// Staging area manager
pub struct StagingArea<'a> {
    repo: &'a Git2Repository,
}

impl<'a> StagingArea<'a> {
    /// Create a new staging area manager
    pub fn new(repo: &'a Git2Repository) -> Self {
        Self { repo }
    }

    /// Get all files with changes (staged and unstaged)
    pub fn get_status(&self) -> Result<Vec<WorkingTreeFile>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            let working_status = self.convert_status(status);
            if let Some(ws) = working_status {
                files.push(WorkingTreeFile {
                    path,
                    status: ws,
                    old_path: None,
                });
            }
        }

        Ok(files)
    }

    /// Get only staged files
    pub fn get_staged(&self) -> Result<Vec<WorkingTreeFile>> {
        Ok(self.get_status()?.into_iter().filter(|f| f.status.is_staged()).collect())
    }

    /// Get only unstaged files (including untracked)
    pub fn get_unstaged(&self) -> Result<Vec<WorkingTreeFile>> {
        Ok(self.get_status()?.into_iter().filter(|f| !f.status.is_staged()).collect())
    }

    /// Stage a file
    pub fn stage_file(&self, path: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        let path = Path::new(path);

        // Check if file exists
        let full_path = self.repo.workdir().unwrap().join(path);
        if full_path.exists() {
            index.add_path(path)?;
        } else {
            // File was deleted, remove from index
            index.remove_path(path)?;
        }

        index.write()?;
        Ok(())
    }

    /// Unstage a file
    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        let head_tree = head.tree()?;

        self.repo.reset_default(Some(&head.into_object()), [Path::new(path)])?;
        Ok(())
    }

    /// Stage all changes
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Unstage all changes
    pub fn unstage_all(&self) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo.reset(
            &head.into_object(),
            git2::ResetType::Mixed,
            None,
        )?;
        Ok(())
    }

    /// Create a commit with the staged changes
    pub fn commit(&self, message: &str) -> Result<String> {
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let signature = self.repo.signature()?;
        let head = self.repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(commit_id.to_string())
    }

    /// Convert git2 status flags to our status type
    fn convert_status(&self, status: Status) -> Option<WorkingTreeStatus> {
        if status.is_index_new() {
            Some(WorkingTreeStatus::StagedNew)
        } else if status.is_index_modified() {
            Some(WorkingTreeStatus::StagedModified)
        } else if status.is_index_deleted() {
            Some(WorkingTreeStatus::StagedDeleted)
        } else if status.is_index_renamed() {
            Some(WorkingTreeStatus::Renamed)
        } else if status.is_wt_new() {
            Some(WorkingTreeStatus::Untracked)
        } else if status.is_wt_modified() {
            Some(WorkingTreeStatus::Modified)
        } else if status.is_wt_deleted() {
            Some(WorkingTreeStatus::Deleted)
        } else if status.is_conflicted() {
            Some(WorkingTreeStatus::Conflicted)
        } else {
            None
        }
    }
}

/// Get the diff for unstaged changes
pub fn get_unstaged_diff(repo: &Git2Repository) -> Result<String> {
    let diff = repo.diff_index_to_workdir(None, None)?;
    format_diff(&diff)
}

/// Get the diff for staged changes
pub fn get_staged_diff(repo: &Git2Repository) -> Result<String> {
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), None, None)?;
    format_diff(&diff)
}

/// Format a diff as a string
fn format_diff(diff: &git2::Diff) -> Result<String> {
    let mut diff_text = String::new();

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            'H' => "",  // Hunk header
            'F' => "",  // File header
            _ => "",
        };

        if let Ok(content) = std::str::from_utf8(line.content()) {
            if !prefix.is_empty() {
                diff_text.push_str(prefix);
            }
            diff_text.push_str(content);
        }

        true
    })?;

    Ok(diff_text)
}
