// src/core/bootloader/mod.rs

use std::io;
use std::path::{Path, PathBuf};

// Re-export the specific implementations
pub mod grub;
pub mod kernelstub;
pub mod systemd_boot;

// Import the specific config types and BootloaderType
use crate::core::system::BootloaderType;
use grub::GrubConfig;
use kernelstub::KernelstubConfig;
use systemd_boot::SystemdBootConfig;

/// Trait for managing bootloader configuration
pub trait BootloaderManager {
    /// Detects the currently active kernel command line parameters from config
    fn get_config_parameters(&self) -> io::Result<Vec<String>>;

    /// Adds kernel parameters to the bootloader configuration file
    /// Returns true if changes were made or would be made (in dry_run).
    fn add_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool>;

    /// Removes kernel parameters from the bootloader configuration file
    /// Returns true if changes were made or would be made (in dry_run).
    fn remove_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool>;

    /// Creates a backup of the relevant configuration file(s)
    fn create_backup(&self) -> io::Result<Vec<PathBuf>>;

    /// Updates the bootloader itself (e.g., runs update-grub)
    fn update_bootloader(&self, dry_run: bool) -> io::Result<()>;
}

/// Factory function to get the appropriate BootloaderManager
pub fn get_bootloader_manager(bootloader_type: &BootloaderType) -> Option<Box<dyn BootloaderManager>> {
    match bootloader_type {
        BootloaderType::Grub => Some(Box::new(GrubConfig::new())),
        BootloaderType::SystemdBoot => Some(Box::new(SystemdBootConfig::new())),
        BootloaderType::PopOsKernelstub => Some(Box::new(KernelstubConfig::new())),
        _ => {
            println!("Warning: Bootloader type {:?} not fully supported yet.", bootloader_type);
            None
        }
    }
}