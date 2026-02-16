//! TUI Layer
//!
//! Responsible for rendering UI widgets, handling input, managing navigation
//! and focus, and updating views based on state changes.

mod event;
mod theme;
mod tui;
mod widgets;

pub use event::Event;
pub use theme::Theme;
pub use tui::Tui;
