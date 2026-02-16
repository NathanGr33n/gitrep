//! Git Data Layer
//!
//! Handles all repository interactions using libgit2 bindings.

mod commit;
mod repository;
mod staging;

pub use commit::CommitInfo;
pub use repository::{BranchInfo, FileChange, FileStatus, Repository};
pub use staging::{get_staged_diff, get_unstaged_diff, StagingArea, WorkingTreeFile, WorkingTreeStatus};
