//! Git Data Layer
//!
//! Handles all repository interactions using libgit2 bindings.

mod commit;
mod repository;

pub use commit::CommitInfo;
pub use repository::Repository;
