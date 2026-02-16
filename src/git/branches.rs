//! Branch Management
//!
//! Handles branch operations including listing, creating, deleting, and switching.

use anyhow::{Context, Result};
use git2::{BranchType, Repository as Git2Repository};

/// Extended branch information with ahead/behind tracking
#[derive(Debug, Clone)]
pub struct BranchDetail {
    /// Branch name
    pub name: String,
    /// Whether this is the current HEAD branch
    pub is_head: bool,
    /// Whether this is a local branch
    pub is_local: bool,
    /// Upstream branch name (if any)
    pub upstream: Option<String>,
    /// Commits ahead of upstream
    pub ahead: usize,
    /// Commits behind upstream
    pub behind: usize,
    /// Latest commit hash on this branch
    pub latest_commit: String,
    /// Latest commit message summary
    pub latest_message: String,
}

/// Tag information
#[derive(Debug, Clone)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Target commit hash
    pub target: String,
    /// Tag message (for annotated tags)
    pub message: Option<String>,
    /// Tagger name (for annotated tags)
    pub tagger: Option<String>,
    /// Whether this is a lightweight tag
    pub is_lightweight: bool,
}

/// Branch manager for operations
pub struct BranchManager<'a> {
    repo: &'a Git2Repository,
}

impl<'a> BranchManager<'a> {
    /// Create a new branch manager
    pub fn new(repo: &'a Git2Repository) -> Self {
        Self { repo }
    }

    /// Get detailed list of all branches
    pub fn get_branches(&self) -> Result<Vec<BranchDetail>> {
        let mut branches = Vec::new();

        // Get local branches
        for branch_result in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            if let Some(detail) = self.branch_to_detail(&branch, true)? {
                branches.push(detail);
            }
        }

        // Get remote branches
        for branch_result in self.repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch_result?;
            if let Some(detail) = self.branch_to_detail(&branch, false)? {
                branches.push(detail);
            }
        }

        // Sort: current branch first, then local branches, then remote
        branches.sort_by(|a, b| {
            if a.is_head != b.is_head {
                return b.is_head.cmp(&a.is_head);
            }
            if a.is_local != b.is_local {
                return b.is_local.cmp(&a.is_local);
            }
            a.name.cmp(&b.name)
        });

        Ok(branches)
    }

    /// Convert a git2 branch to BranchDetail
    fn branch_to_detail(
        &self,
        branch: &git2::Branch,
        is_local: bool,
    ) -> Result<Option<BranchDetail>> {
        let name = match branch.name()? {
            Some(n) => n.to_string(),
            None => return Ok(None),
        };

        let is_head = branch.is_head();

        // Get upstream info
        let (upstream, ahead, behind) = if is_local {
            self.get_upstream_info(branch)?
        } else {
            (None, 0, 0)
        };

        // Get latest commit info
        let reference = branch.get();
        let commit = reference.peel_to_commit()?;
        let latest_commit = commit.id().to_string()[..7].to_string();
        let latest_message = commit
            .summary()
            .unwrap_or("(no message)")
            .to_string();

        Ok(Some(BranchDetail {
            name,
            is_head,
            is_local,
            upstream,
            ahead,
            behind,
            latest_commit,
            latest_message,
        }))
    }

    /// Get upstream tracking information
    fn get_upstream_info(
        &self,
        branch: &git2::Branch,
    ) -> Result<(Option<String>, usize, usize)> {
        let upstream = match branch.upstream() {
            Ok(u) => u,
            Err(_) => return Ok((None, 0, 0)),
        };

        let upstream_name = upstream
            .name()?
            .map(|s| s.to_string());

        // Calculate ahead/behind
        let local_oid = branch.get().target().unwrap();
        let upstream_oid = upstream.get().target().unwrap();

        let (ahead, behind) = self.repo.graph_ahead_behind(local_oid, upstream_oid)?;

        Ok((upstream_name, ahead, behind))
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str, from_head: bool) -> Result<()> {
        let commit = if from_head {
            self.repo.head()?.peel_to_commit()?
        } else {
            // Default to HEAD
            self.repo.head()?.peel_to_commit()?
        };

        self.repo.branch(name, &commit, false)?;
        Ok(())
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str, force: bool) -> Result<()> {
        let mut branch = self.repo.find_branch(name, BranchType::Local)?;

        if branch.is_head() {
            anyhow::bail!("Cannot delete the current branch");
        }

        if force {
            branch.delete()?;
        } else {
            // Check if branch is fully merged
            let head = self.repo.head()?.peel_to_commit()?;
            let branch_commit = branch.get().peel_to_commit()?;

            if !self.repo.graph_descendant_of(head.id(), branch_commit.id())? {
                anyhow::bail!("Branch '{}' is not fully merged. Use force delete.", name);
            }

            branch.delete()?;
        }

        Ok(())
    }

    /// Switch to a branch
    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let branch = self.repo.find_branch(name, BranchType::Local)?;
        let reference = branch.get();
        let refname = reference.name().context("Invalid branch name")?;

        self.repo.set_head(refname)?;
        self.repo.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                .safe()
                .recreate_missing(true),
        ))?;

        Ok(())
    }

    /// Get all tags
    pub fn get_tags(&self) -> Result<Vec<TagInfo>> {
        let mut tags = Vec::new();

        self.repo.tag_foreach(|oid, name| {
            let name_str = String::from_utf8_lossy(name)
                .trim_start_matches("refs/tags/")
                .to_string();

            // Try to get tag object (for annotated tags)
            let tag_info = if let Ok(tag) = self.repo.find_tag(oid) {
                TagInfo {
                    name: name_str,
                    target: tag.target_id().to_string()[..7].to_string(),
                    message: tag.message().map(|s| s.to_string()),
                    tagger: tag.tagger().and_then(|t| t.name().map(|s| s.to_string())),
                    is_lightweight: false,
                }
            } else {
                // Lightweight tag
                TagInfo {
                    name: name_str,
                    target: oid.to_string()[..7].to_string(),
                    message: None,
                    tagger: None,
                    is_lightweight: true,
                }
            };

            tags.push(tag_info);
            true
        })?;

        // Sort by name
        tags.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(tags)
    }

    /// Create a new tag
    pub fn create_tag(&self, name: &str, message: Option<&str>) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;

        if let Some(msg) = message {
            // Annotated tag
            let sig = self.repo.signature()?;
            self.repo.tag(name, head.as_object(), &sig, msg, false)?;
        } else {
            // Lightweight tag
            self.repo.tag_lightweight(name, head.as_object(), false)?;
        }

        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        self.repo.tag_delete(name)?;
        Ok(())
    }
}
