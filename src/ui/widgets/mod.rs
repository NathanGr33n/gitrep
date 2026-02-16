//! UI Widgets
//!
//! Rendering functions for various UI components.

mod branches;
mod commit_detail;
mod commit_list;
mod header;
mod help;
mod staging;
mod status_bar;

pub use branches::{render_branch_browser, BranchBrowserState, BranchOperation, BranchViewTab};
pub use commit_detail::{render_commit_detail, render_commit_detail_full};
pub use commit_list::render_commit_list;
pub use header::render_header;
pub use help::render_help;
pub use staging::{render_commit_input, render_staging, StagingState};
pub use status_bar::render_status_bar;
