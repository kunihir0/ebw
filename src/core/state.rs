// State Tracking System for Exliar VFIO Automation Framework
//
// This module handles tracking changes made to the system,
// allowing for potential rollback or cleanup operations.

use std::fs;
use std::io; // Removed unused Write import
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

// Assume PciDevice is accessible, e.g., from crate::gpu::detection
// We might need to adjust imports based on actual project structure
// use crate::gpu::detection::PciDevice;
// We'll need VfioManager eventually to call its unbind/bind methods
// use crate::core::vfio::VfioManager;
// We'll need BootloaderConfig eventually
// use crate::core::bootloader::BootloaderConfig;

/// Represents a single change made to the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Change {
    /// A file was modified, backup created at the specified path
    FileModified { path: PathBuf, backup_path: PathBuf },
    /// A kernel parameter was added (or modified)
    KernelParamAdded { parameter: String, bootloader: String },
    /// A kernel parameter was removed (we need original value to restore?)
    KernelParamRemoved { parameter: String, bootloader: String, original_value: Option<String> },
    /// A kernel module was configured to load (e.g., in /etc/modules-load.d)
    ModuleLoaded { name: String, config_path: PathBuf, backup_path: Option<PathBuf> },
    /// A device driver binding was changed (bound to a new driver)
    DriverBound { device_bdf: String, new_driver: String, original_driver: Option<String> },
    /// A device driver was unbound (original driver might be needed to rebind)
    DriverUnbound { device_bdf: String, original_driver: Option<String> },
    // Add other change types as needed (e.g., ServiceStarted, DirectoryCreated)
}

/// Tracks the sequence of changes made during configuration
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StateTracker {
    changes: Vec<Change>,
    #[serde(skip)] // Don't serialize the state file path itself
    state_file_path: PathBuf,
}

impl StateTracker {
    /// Creates a new StateTracker, optionally loading from a state file
    pub fn new(state_file_path: PathBuf) -> io::Result<Self> {
        if state_file_path.exists() {
            match Self::load_state(&state_file_path) {
                Ok(tracker) => Ok(tracker),
                Err(e) => {
                    eprintln!("Warning: Failed to load existing state file at {}: {}. Starting fresh.", state_file_path.display(), e);
                    // If loading fails, start with a fresh state but keep the path
                    Ok(Self {
                        changes: Vec::new(),
                        state_file_path,
                    })
                }
            }
        } else {
            Ok(Self {
                changes: Vec::new(),
                state_file_path,
            })
        }
    }

    /// Records a new change and saves the state
    pub fn record_change(&mut self, change: Change) -> io::Result<()> {
        println!("Recording change: {:?}", change); // Basic logging
        self.changes.push(change);
        self.save_state() // Save state after every change
    }

    /// Saves the current state to the specified file
    pub fn save_state(&self) -> io::Result<()> {
        let serialized_state = serde_json::to_string_pretty(&self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to serialize state: {}", e)))?;

        // Ensure directory exists
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.state_file_path, serialized_state)?;
        // println!("State saved to {}", self.state_file_path.display()); // Reduce noise
        Ok(())
    }

    /// Loads state from the specified file
    fn load_state(path: &Path) -> io::Result<Self> {
        if !path.exists() {
             return Err(io::Error::new(io::ErrorKind::NotFound, "State file not found"));
        }
        let serialized_state = fs::read_to_string(path)?;
        let mut tracker: Self = serde_json::from_str(&serialized_state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize state: {}", e)))?;

        // Restore the non-serialized path
        tracker.state_file_path = path.to_path_buf();
        println!("State loaded from {}", path.display());
        Ok(tracker)
    }

    /// Attempts to rollback all recorded changes in reverse order.
    /// Note: This is a complex operation and might not fully succeed,
    /// especially for bootloader/kernel parameter changes which often require a reboot.
    pub fn rollback_all(&mut self /* Add necessary context like VfioManager, BootloaderManager */) -> io::Result<()> {
        println!("Attempting to rollback recorded changes...");
        let mut rollback_errors = Vec::new();

        // Iterate changes in reverse order to undo them
        while let Some(change) = self.changes.pop() {
            println!("Rolling back: {:?}", change);
            match self.undo_change(change) { // Pass self if undo_change needs it
                Ok(_) => println!("  Rollback successful."),
                Err(e) => {
                    let err_msg = format!("  Rollback failed: {}", e);
                    println!("{}", err_msg);
                    rollback_errors.push(err_msg);
                    // Decide whether to continue or stop on error?
                    // For now, let's try to continue rolling back other changes.
                }
            }
        }

        // Save the (now likely empty) state after rollback attempts
        self.save_state()?;

        if rollback_errors.is_empty() {
            println!("Rollback process completed.");
            // Optionally remove the state file if empty?
            // if self.changes.is_empty() {
            //     let _ = fs::remove_file(&self.state_file_path);
            // }
            Ok(())
        } else {
            let combined_errors = rollback_errors.join("\n");
            let final_err_msg = format!("Rollback process completed with errors:\n{}", combined_errors);
            println!("{}", final_err_msg);
            // Consider generating cleanup script as fallback here too?
            Err(io::Error::new(io::ErrorKind::Other, final_err_msg))
        }
    }

    /// Attempts to undo a single change.
    // Make this take &self if it doesn't modify the tracker itself,
    // or &mut self if it needs to modify something (unlikely for undo logic itself).
    fn undo_change(&self, change: Change /* Add context args */) -> io::Result<()> {
        match change {
            Change::FileModified { path, backup_path } => {
                if backup_path.exists() {
                    println!("  Restoring backup {} to {}", backup_path.display(), path.display());
                    // Use rename for atomic move if possible, overwrite destination
                    fs::rename(&backup_path, &path)?;
                } else {
                    let msg = format!("Backup file {} not found, cannot restore {}", backup_path.display(), path.display());
                    println!("  Warning: {}", msg);
                    // Decide if this is an error or just a warning
                    // return Err(io::Error::new(io::ErrorKind::NotFound, msg));
                }
            },
            Change::KernelParamAdded { parameter, bootloader } => {
                println!("  Manual action needed: Remove kernel parameter '{}' for {} bootloader and update.", parameter, bootloader);
                // Requires integration with BootloaderManager::remove_parameter(...)
                // This is complex and likely needs user confirmation + reboot.
            },
            Change::KernelParamRemoved { parameter, bootloader, original_value } => {
                 println!("  Manual action needed: Restore kernel parameter '{}' (original value: {:?}) for {} bootloader and update.", parameter, original_value, bootloader);
                 // Requires integration with BootloaderManager::add_parameter(...)
            },
            Change::ModuleLoaded { name: _, config_path, backup_path } => {
                 println!("  Attempting to undo module load configuration...");
                 if let Some(bp) = backup_path {
                     if bp.exists() {
                         println!("  Restoring backup {} to {}", bp.display(), config_path.display());
                         fs::rename(&bp, &config_path)?;
                     } else {
                          println!("  Backup for {} not found, attempting to remove config file.", config_path.display());
                          let _ = fs::remove_file(&config_path); // Try removing if no backup
                     }
                 } else if config_path.exists() {
                     // If no backup was recorded, maybe just remove the file we created?
                     println!("  No backup recorded for {}, removing file.", config_path.display());
                     fs::remove_file(&config_path)?;
                 }
            },
            Change::DriverBound { device_bdf, new_driver: _, original_driver } => {
                 println!("  Attempting to rebind device {} to original driver ({:?})", device_bdf, original_driver);
                 if let Some(_orig_driver) = original_driver { // Mark as unused for now
                     // Requires integration with VfioManager or similar to rebind
                     // For now, just print message
                     println!("  Manual action needed or integrate with driver binding logic.");
                     // Example (needs VfioManager instance):
                     // let vfio_manager = VfioManager::new(...); // Need SystemInfo
                     // let dummy_device = PciDevice { bdf: device_bdf, ... }; // Need a way to get PciDevice info
                     // vfio_manager.unbind_device(&dummy_device, false)?; // Unbind from vfio-pci
                     // vfio_manager.bind_device_to_driver(&dummy_device, &orig_driver)?; // Bind back
                 } else {
                     println!("  Original driver for {} unknown, cannot automatically rebind.", device_bdf);
                 }
            },
            Change::DriverUnbound { device_bdf, original_driver } => {
                 println!("  Attempting to rebind device {} to original driver ({:?})", device_bdf, original_driver);
                 if let Some(_orig_driver) = original_driver { // Mark as unused for now
                     // Requires integration with VfioManager or similar to rebind
                     println!("  Manual action needed or integrate with driver binding logic.");
                     // Example (needs VfioManager instance):
                     // let vfio_manager = VfioManager::new(...);
                     // let dummy_device = PciDevice { bdf: device_bdf, ... };
                     // vfio_manager.bind_device_to_driver(&dummy_device, &orig_driver)?;
                 } else {
                     println!("  Original driver for {} unknown, cannot automatically rebind.", device_bdf);
                 }
            },
            // Handle other change types...
        }
        Ok(())
    }


    /// Generates a cleanup script based on recorded changes.
    pub fn generate_cleanup_script(&self) -> io::Result<String> {
        println!("Generating cleanup script...");
        let mut script_content = String::new();
        script_content.push_str("#!/bin/bash\n");
        script_content.push_str("# Auto-generated cleanup script by Exliar VFIO\n");
        script_content.push_str("# Run this script with sudo to attempt reverting changes.\n");
        script_content.push_str("# Warning: This script is basic and might not fully revert all changes.\n\n");
        script_content.push_str("set -e # Exit on error\n\n");

        for change in self.changes.iter().rev() { // Iterate in reverse for cleanup
            match change {
                Change::FileModified { path, backup_path } => {
                    script_content.push_str(&format!(
                        "# Restore file {}\n", path.display()
                    ));
                    script_content.push_str(&format!(
                        "if [ -f \"{}\" ]; then\n", backup_path.display()
                    ));
                    script_content.push_str(&format!(
                        "  echo \"Restoring backup {} to {}\"\n", backup_path.display(), path.display()
                    ));
                    // Use cp and rm for safety instead of mv -f
                    script_content.push_str(&format!(
                        "  cp -f \"{}\" \"{}\" && rm -f \"{}\" || echo \"Error restoring file\"\n",
                         backup_path.display(), path.display(), backup_path.display()
                    ));
                     script_content.push_str("else\n");
                     script_content.push_str(&format!(
                        "  echo \"Backup file {} not found, cannot restore\"\n", backup_path.display()
                    ));
                    script_content.push_str("fi\n\n");
                },
                Change::KernelParamAdded { parameter, bootloader } => {
                     script_content.push_str(&format!(
                        "# Remove kernel parameter '{}' for {}\n", parameter, bootloader
                    ));
                     script_content.push_str(&format!(
                        "echo \"Manual action needed: Remove kernel parameter '{}' for {} bootloader and update\"\n\n", parameter, bootloader
                    ));
                },
                Change::KernelParamRemoved { parameter, bootloader, original_value } => {
                     script_content.push_str(&format!(
                        "# Restore kernel parameter '{}' (original: {:?}) for {}\n", parameter, original_value, bootloader
                    ));
                     script_content.push_str(&format!(
                        "echo \"Manual action needed: Restore kernel parameter '{}' (original: {:?}) for {} bootloader and update\"\n\n", parameter, original_value, bootloader
                    ));
                },
                 Change::ModuleLoaded { name, config_path, backup_path } => {
                     script_content.push_str(&format!(
                        "# Revert module load config for '{}' at {}\n", name, config_path.display()
                    ));
                     if let Some(bp) = backup_path {
                         script_content.push_str(&format!(
                            "if [ -f \"{}\" ]; then\n", bp.display()
                        ));
                         script_content.push_str(&format!(
                            "  echo \"Restoring backup {} to {}\"\n", bp.display(), config_path.display()
                        ));
                         script_content.push_str(&format!(
                            "  mv -f \"{}\" \"{}\" || echo \"Error restoring module config backup\"\n", bp.display(), config_path.display()
                        ));
                         script_content.push_str("else\n");
                         script_content.push_str(&format!(
                            "  echo \"Backup file {} not found, removing config file {}\"\n", bp.display(), config_path.display()
                        ));
                         script_content.push_str(&format!(
                            "  rm -f \"{}\" || echo \"Error removing module config file\"\n", config_path.display()
                        ));
                         script_content.push_str("fi\n\n");
                     } else {
                          script_content.push_str(&format!(
                            "# No backup recorded for {}, removing file {}\n", name, config_path.display()
                        ));
                          script_content.push_str(&format!(
                            "rm -f \"{}\" || echo \"Error removing module config file\"\n\n", config_path.display()
                        ));
                     }
                 },
                 Change::DriverBound { device_bdf, new_driver: _, original_driver } | // Combine logic
                 Change::DriverUnbound { device_bdf, original_driver } => {
                     script_content.push_str(&format!(
                        "# Rebind device {} to its original driver ({:?})\n", device_bdf, original_driver
                    ));
                     if let Some(driver) = original_driver {
                         script_content.push_str(&format!(
                            "echo \"Attempting to rebind {} to driver {}\"\n", device_bdf, driver
                        ));
                         // Simplified rebind attempt - might not always work
                         script_content.push_str("# Clear override first\n");
                         script_content.push_str(&format!(
                            "echo -n > \"/sys/bus/pci/devices/{}/driver_override\" 2>/dev/null\n", device_bdf.replace(':', "/")
                         ));
                         script_content.push_str("# Unbind from current (likely vfio-pci)\n");
                         script_content.push_str(&format!(
                            "echo \"{}\" > \"/sys/bus/pci/drivers/vfio-pci/unbind\" 2>/dev/null\n", device_bdf
                         ));
                         script_content.push_str("# Re-probe to let kernel pick original driver\n");
                         script_content.push_str(&format!(
                            "echo \"{}\" > /sys/bus/pci/drivers_probe 2>/dev/null || echo \"Failed to trigger re-probe for {}\"\n\n", device_bdf, device_bdf
                        ));
                     } else {
                         script_content.push_str(&format!(
                            "# Original driver for {} unknown, cannot automatically rebind. May need manual rebind or reboot.\n", device_bdf
                        ));
                         script_content.push_str(&format!(
                            "echo \"Original driver for {} unknown, cannot automatically rebind. May need manual rebind or reboot.\"\n\n", device_bdf
                        ));
                     }
                 },
                // Add cases for other Change types here...
                // _ => {
                //     script_content.push_str(&format!("# Cleanup action for {:?} not implemented\n\n", change));
                // }
            }
        }

        script_content.push_str("echo 'Cleanup script finished.'\n");
        Ok(script_content)
    }

    /// Clears all recorded changes and saves the empty state
    pub fn clear_state(&mut self) -> io::Result<()> {
        println!("Clearing recorded state...");
        self.changes.clear();
        self.save_state()
    }
}