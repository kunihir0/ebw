// Exliar VFIO Automation Framework
//
// A modular, extensible system for automating VFIO GPU passthrough and VM management

// Core functionality modules
pub mod core;

// GPU management modules
pub mod gpu;

// Plugin system
pub mod plugin;

// User interface
pub mod ui;

// Utility functions
pub mod utils;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");