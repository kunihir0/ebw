// Utility functions for Exliar VFIO Automation Framework

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Basic logging utilities
pub mod logging {
    /// Log info level message
    pub fn info(message: &str) {
        println!("INFO: {}", message);
    }

    /// Log success message
    pub fn success(message: &str) {
        println!("SUCCESS: {}", message);
    }

    /// Log warning message
    pub fn warning(message: &str) {
        println!("WARNING: {}", message);
    }

    /// Log error message
    pub fn error(message: &str) {
        eprintln!("ERROR: {}", message);
    }

    /// Log debug message if debug is enabled
    pub fn debug(message: &str, debug_enabled: bool) {
        if debug_enabled {
            println!("DEBUG: {}", message);
        }
    }
}

/// Helper function to run a system command and return the output
pub fn run_command(command: &str) -> Result<String, String> {
    use std::process::Command;
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    
    let program = parts[0];
    let args = &parts[1..];
    
    match Command::new(program).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                Err(format!("Command failed: {}", stderr))
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Helper to create a timestamped backup of a file
pub fn create_timestamped_backup(file_path: &Path) -> io::Result<PathBuf> {
    if !file_path.exists() {
        // No need to backup if file doesn't exist
        return Ok(file_path.to_path_buf()); // Return original path conceptually
    }

    // Use chrono if available, otherwise fallback to a simpler timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    // Fallback (if chrono is not added):
    // let timestamp = std::time::SystemTime::now()
    //     .duration_since(std::time::UNIX_EPOCH)
    //     .map(|d| d.as_secs().to_string())
    //     .unwrap_or_else(|_| "timestamp_error".to_string());

    let backup_filename = format!("{}.backup_{}",
        file_path.file_name().unwrap_or_default().to_string_lossy(),
        timestamp);
    let backup_path = file_path.with_file_name(backup_filename);

    fs::copy(file_path, &backup_path)?;
    Ok(backup_path)
}