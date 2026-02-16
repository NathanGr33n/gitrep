//! Event Types
//!
//! Defines the event types used by the application.

use crossterm::event::{KeyEvent, MouseEvent};

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),
}
