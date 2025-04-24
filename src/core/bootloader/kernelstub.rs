// src/core/bootloader/kernelstub.rs

use std::io;
use std::path::PathBuf;
use std::process::Command; // Needed for running kernelstub commands

use super::BootloaderManager; // Import the trait from the parent module

/// Pop!_OS kernelstub manager
#[derive(Debug)] // Added Debug derive
pub struct KernelstubConfig {
    // Path to kernelstub binary? Might not be needed if it's in PATH
    // We might need to store the parsed parameters if get_config_parameters is implemented fully.
}

impl KernelstubConfig {
     pub fn new() -> Self { Self {} }

     // Helper function to run kernelstub commands
     fn run_kernelstub(&self, args: &[&str], dry_run: bool) -> io::Result<()> {
         let command_str = format!("sudo kernelstub {}", args.join(" "));
         if dry_run {
             println!("[DRY RUN] Would execute: {}", command_str);
             return Ok(());
         }

         println!("Executing: {}", command_str);
         let status = Command::new("sudo")
             .arg("kernelstub")
             .args(args)
             .status()?;

         if status.success() {
             Ok(())
         } else {
             let err_msg = format!("kernelstub command failed: {:?} with exit code: {:?}", args, status.code());
             println!("Error: {}", err_msg);
             Err(io::Error::new(io::ErrorKind::Other, err_msg))
         }
     }
}

impl BootloaderManager for KernelstubConfig {
     fn get_config_parameters(&self) -> io::Result<Vec<String>> {
         println!("Warning: kernelstub get_config_parameters not fully implemented.");
         // Need to run `sudo kernelstub -p` and parse the "Kernel Boot Options" line
         let output = Command::new("sudo")
             .arg("kernelstub")
             .arg("-p") // Print current config
             .output()?;

         if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Err(io::Error::new(io::ErrorKind::Other, format!("kernelstub -p failed: {}", stderr)));
         }

         let stdout = String::from_utf8_lossy(&output.stdout);
         // Example line: "Kernel Boot Options: quiet loglevel=0 systemd.show_status=false splash"
         for line in stdout.lines() {
             if let Some(options_part) = line.strip_prefix("Kernel Boot Options:") {
                 return Ok(options_part.trim().split_whitespace().map(String::from).collect());
             }
         }

         Ok(Vec::new()) // Return empty if line not found
     }

     fn add_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Adding parameters {:?} using kernelstub...", params);
         let mut changed = false; // Track if any command succeeds or would run
         for param in params {
             // kernelstub -a adds one parameter at a time
             match self.run_kernelstub(&["-a", param], dry_run) {
                 Ok(_) => changed = true, // Assume change happened if command ran/would run
                 Err(e) => {
                     // Decide if we should continue or return the error
                     // For now, let's print the error and continue
                     eprintln!("Failed to add parameter '{}' with kernelstub: {}", param, e);
                 }
             }
         }
         Ok(changed) // Return true if any add command was attempted/successful
     }

     fn remove_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Removing parameters {:?} using kernelstub...", params);
         let mut changed = false;
         for param in params {
             // kernelstub -d removes one parameter at a time
             match self.run_kernelstub(&["-d", param], dry_run) {
                 Ok(_) => changed = true,
                 Err(e) => {
                     eprintln!("Failed to remove parameter '{}' with kernelstub: {}", param, e);
                 }
             }
         }
         Ok(changed) // Return true if any remove command was attempted/successful
     }

     fn create_backup(&self) -> io::Result<Vec<PathBuf>> {
         println!("Info: kernelstub manages its own configuration state; explicit backup not typically needed via this tool.");
         // kernelstub might have internal backups, but we don't manage them directly here.
         Ok(Vec::new())
     }

     fn update_bootloader(&self, _dry_run: bool) -> io::Result<()> {
         // kernelstub applies changes immediately when -a or -d is used.
         // There might be a separate update command, but typically not needed for param changes.
         println!("kernelstub configuration updated (changes applied directly via add/remove).");
         Ok(())
     }
}