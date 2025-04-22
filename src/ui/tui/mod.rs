// TUI implementation using Ratatui
// This module contains the main application loop, state management,
// and rendering functions for the terminal user interface.

// Re-export the main run function
pub use app::run_app;

mod app;
pub mod render; // Make render module public within tui module
mod state;
mod input;