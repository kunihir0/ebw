// Bootloader Configuration Module for Exliar VFIO Automation Framework
//
// This module handles detecting the bootloader and managing kernel parameters.

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex; // Import Regex

// Assuming BootloaderType is pub in system module
use crate::core::system::BootloaderType;
// Import create_timestamped_backup from the crate's utils module
use crate::utils::create_timestamped_backup;

/// Trait for managing bootloader configuration
pub trait BootloaderManager {
    /// Detects the currently active kernel command line parameters from config
    fn get_config_parameters(&self) -> io::Result<Vec<String>>;

    /// Adds kernel parameters to the bootloader configuration file
    fn add_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool>; // Returns true if changed

    /// Removes kernel parameters from the bootloader configuration file
    fn remove_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool>; // Returns true if changed

    /// Creates a backup of the relevant configuration file(s)
    fn create_backup(&self) -> io::Result<Vec<PathBuf>>;

    /// Updates the bootloader itself (e.g., runs update-grub)
    fn update_bootloader(&self, dry_run: bool) -> io::Result<()>;
}

/// GRUB bootloader configuration manager
pub struct GrubConfig {
    default_grub_path: PathBuf,
    // We might need SystemInfo here later to determine the correct update command
}

impl GrubConfig {
    pub fn new() -> Self {
        Self { default_grub_path: PathBuf::from("/etc/default/grub") }
    }

    /// Reads the GRUB_CMDLINE_LINUX_DEFAULT parameters from the config file
    fn read_grub_cmdline(&self) -> io::Result<String> {
        if !self.default_grub_path.exists() {
            // If the file doesn't exist, treat it as empty parameters
            return Ok(String::new());
        }
        let content = fs::read_to_string(&self.default_grub_path)?;
        // Regex to find the line, handling different quote types and spacing
        let re = Regex::new(r#"^\s*GRUB_CMDLINE_LINUX_DEFAULT\s*=\s*(["'])(.*?)\1"#)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Regex error: {}", e)))?;

        if let Some(caps) = re.captures(&content) {
            // Group 2 contains the parameters within the quotes
            Ok(caps.get(2).map_or(String::new(), |m| m.as_str().to_string()))
        } else {
            // If the line is not found, return empty string
            Ok(String::new())
        }
    }

     /// Writes the new GRUB_CMDLINE_LINUX_DEFAULT parameters to the config file
     fn write_grub_cmdline(&self, new_params_str: &str) -> io::Result<()> {
        if !self.default_grub_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "/etc/default/grub not found"));
        }
        let mut content = fs::read_to_string(&self.default_grub_path)?;
        // Regex to capture the prefix (including quotes) and the existing params, multiline flag needed
        // Corrected Regex: (?m) flag inside the string literal
        let re = Regex::new(r#"(?m)^(?P<prefix>\s*GRUB_CMDLINE_LINUX_DEFAULT\s*=\s*["'])(?P<params>.*?)(?P<suffix>["']\s*)$"#)
             .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Regex error: {}", e)))?;

        if re.is_match(&content) {
             // Replace the existing parameters within the quotes
             content = re.replace(&content, format!("${{prefix}}{}${{suffix}}", new_params_str)).to_string();
        } else {
             // If line doesn't exist or doesn't match format, append it (basic approach)
             println!("Warning: GRUB_CMDLINE_LINUX_DEFAULT line not found or malformed. Appending.");
             content.push_str(&format!("\nGRUB_CMDLINE_LINUX_DEFAULT=\"{}\"\n", new_params_str));
        }

        // Write the modified content back to the file
        let mut file = fs::File::create(&self.default_grub_path)?;
        file.write_all(content.as_bytes())
    }
}

impl BootloaderManager for GrubConfig {
    fn get_config_parameters(&self) -> io::Result<Vec<String>> {
        let cmdline = self.read_grub_cmdline()?;
        Ok(cmdline.split_whitespace().map(String::from).collect())
    }

    fn add_parameters(&mut self, params_to_add: &[&str], dry_run: bool) -> io::Result<bool> {
        println!("Adding parameters {:?} to GRUB config...", params_to_add);
        let current_params_str = self.read_grub_cmdline()?;
        // Use HashSet for efficient checking and modification
        let mut current_params: HashSet<String> =
            current_params_str.split_whitespace().map(String::from).collect();

        let mut changed = false;
        for param_str in params_to_add {
            let param = param_str.to_string();
            // Check if the parameter key already exists (e.g., "iommu=")
            let key = param.split('=').next().unwrap_or(&param);
            let key_prefix = format!("{}=", key);

            // Remove any existing parameter with the same key or exact flag match
            let initial_len = current_params.len();
            current_params.retain(|p| !p.starts_with(&key_prefix) && p != key);

            // Add the new parameter
            // Check if insertion actually happened or if it replaced a removed one
            if current_params.insert(param) || current_params.len() != initial_len {
                 changed = true;
            }
        }

        if changed {
             let mut final_params: Vec<String> = current_params.into_iter().collect();
             final_params.sort(); // Sort for consistency
             let final_params_str = final_params.join(" ");
             println!("  New GRUB_CMDLINE_LINUX_DEFAULT: \"{}\"", final_params_str);

             if dry_run {
                 println!("[DRY RUN] Would modify {}", self.default_grub_path.display());
             } else {
                 self.create_backup()?; // Backup before writing
                 self.write_grub_cmdline(&final_params_str)?;
                 println!("  Successfully updated {}", self.default_grub_path.display());
                 // Note: update_bootloader() needs to be called separately
             }
             Ok(true) // Indicate that changes were made/would be made
        } else {
            println!("  Parameters already present or no changes needed.");
            Ok(false) // Indicate no changes were made
        }
    }

    fn remove_parameters(&mut self, params_to_remove: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Removing parameters {:?} from GRUB config...", params_to_remove);
         let current_params_str = self.read_grub_cmdline()?;
         let mut current_params: HashSet<String> =
             current_params_str.split_whitespace().map(String::from).collect();

        let mut changed = false;
        for param_to_remove_str in params_to_remove {
             let param_to_remove = param_to_remove_str.to_string();
             // Remove exact match or key=value match
             let key_to_remove = param_to_remove.split('=').next().unwrap_or(&param_to_remove);
             let initial_len = current_params.len();

             // Retain only parameters that DO NOT match the key or the exact string
             current_params.retain(|p| {
                 let p_key = p.split('=').next().unwrap_or(p);
                 p != &param_to_remove && p_key != key_to_remove
             });

             // Check if anything was actually removed
             if current_params.len() < initial_len {
                 changed = true;
             }
        }

        if changed {
            let mut final_params: Vec<String> = current_params.into_iter().collect();
            final_params.sort();
            let final_params_str = final_params.join(" ");
            println!("  New GRUB_CMDLINE_LINUX_DEFAULT: \"{}\"", final_params_str);

            if dry_run {
                 println!("[DRY RUN] Would modify {}", self.default_grub_path.display());
            } else {
                self.create_backup()?;
                self.write_grub_cmdline(&final_params_str)?;
                println!("  Successfully updated {}", self.default_grub_path.display());
                // Note: update_bootloader() needs to be called separately
            }
            Ok(true) // Indicate changes were made/would be made
        } else {
             println!("  Parameters not found or already removed.");
             Ok(false) // Indicate no changes were made
        }
    }

    fn create_backup(&self) -> io::Result<Vec<PathBuf>> {
        let backup_path = create_timestamped_backup(&self.default_grub_path)?; // Use imported function
        println!("Created backup: {}", backup_path.display());
        Ok(vec![backup_path])
    }

    fn update_bootloader(&self, dry_run: bool) -> io::Result<()> {
        // Determine correct update command based on system (needs integration with SystemInfo)
        // This requires passing SystemInfo or DistroFamily to GrubConfig or this method
        // For now, using placeholders based on common paths
        let update_cmd_str = if Path::new("/usr/bin/update-grub").exists() {
            "sudo update-grub" // Debian/Ubuntu
        } else if Path::new("/usr/sbin/grub2-mkconfig").exists() {
            // Check common output paths for Fedora/RHEL/SUSE
            if Path::new("/boot/efi/EFI/fedora/grub.cfg").exists() {
                 "sudo grub2-mkconfig -o /boot/efi/EFI/fedora/grub.cfg"
            } else if Path::new("/boot/grub2/grub.cfg").exists() {
                 "sudo grub2-mkconfig -o /boot/grub2/grub.cfg"
            } else {
                 // Fallback or error
                 println!("Warning: Found grub2-mkconfig but couldn't determine output path.");
                 return Ok(());
            }
        } else if Path::new("/usr/bin/grub-mkconfig").exists() {
             "sudo grub-mkconfig -o /boot/grub/grub.cfg" // Arch
        } else {
            println!("Warning: Could not find standard GRUB update command (update-grub, grub-mkconfig, grub2-mkconfig).");
            return Err(io::Error::new(io::ErrorKind::NotFound, "GRUB update command not found"));
        };

        if dry_run {
            println!("[DRY RUN] Would execute: {}", update_cmd_str);
            return Ok(());
        }

        println!("Executing: {}", update_cmd_str);
        let parts: Vec<&str> = update_cmd_str.split_whitespace().collect();
        let status = Command::new(parts[0])
            .args(&parts[1..])
            .status()?;

        if status.success() {
            println!("GRUB configuration updated successfully.");
            Ok(())
        } else {
            let err_msg = format!("GRUB update command failed with exit code: {:?}", status.code());
            println!("Error: {}", err_msg);
            Err(io::Error::new(io::ErrorKind::Other, err_msg))
        }
    }
}

// --- Placeholders for other bootloaders ---

/// systemd-boot configuration manager
pub struct SystemdBootConfig {
    // Path to ESP, e.g., /boot or /boot/efi or /efi
    esp_path: PathBuf,
    // Path to loader entries, relative to ESP? e.g., loader/entries/
    entries_path: PathBuf,
}

impl SystemdBootConfig {
     // TODO: Implement logic to find ESP and entries path
     pub fn new() -> Self {
         Self {
             esp_path: PathBuf::from("/boot/efi"), // Common default, but needs detection
             entries_path: PathBuf::from("loader/entries"),
         }
     }
}

impl BootloaderManager for SystemdBootConfig {
     fn get_config_parameters(&self) -> io::Result<Vec<String>> {
         println!("Warning: systemd-boot get_config_parameters not fully implemented.");
         // Need to parse all .conf files in entries_path
         Ok(Vec::new())
     }
     fn add_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Warning: systemd-boot add_parameters not fully implemented.");
         println!("  Would add {:?} to relevant entry files under {}", params, self.esp_path.join(&self.entries_path).display());
         if dry_run { Ok(true) } else { Ok(false) } // Placeholder
     }
     fn remove_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Warning: systemd-boot remove_parameters not fully implemented.");
         println!("  Would remove {:?} from relevant entry files under {}", params, self.esp_path.join(&self.entries_path).display());
          if dry_run { Ok(true) } else { Ok(false) } // Placeholder
     }
     fn create_backup(&self) -> io::Result<Vec<PathBuf>> {
         println!("Warning: systemd-boot backup not fully implemented.");
         // Need to backup all relevant .conf files
         Ok(Vec::new())
      }
     fn update_bootloader(&self, _dry_run: bool) -> io::Result<()> {
         // systemd-boot usually doesn't require an explicit update command after editing entries
         println!("systemd-boot configuration updated (no explicit update command needed).");
         Ok(())
     }
}

/// Pop!_OS kernelstub manager
pub struct KernelstubConfig {
    // Path to kernelstub binary? Might not be needed if it's in PATH
}

impl KernelstubConfig {
     pub fn new() -> Self { Self {} }
}

impl BootloaderManager for KernelstubConfig {
     fn get_config_parameters(&self) -> io::Result<Vec<String>> {
         println!("Warning: kernelstub get_config_parameters not fully implemented.");
         // Need to run `kernelstub -p` and parse output
         Ok(Vec::new())
     }
     fn add_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Warning: kernelstub add_parameters not fully implemented.");
         println!("  Would run `sudo kernelstub -a \"param\"` for each param in {:?}", params);
         if dry_run { Ok(true) } else { Ok(false) } // Placeholder
     }
     fn remove_parameters(&mut self, params: &[&str], dry_run: bool) -> io::Result<bool> {
         println!("Warning: kernelstub remove_parameters not fully implemented.");
         println!("  Would run `sudo kernelstub -d \"param\"` for each param in {:?}", params);
         if dry_run { Ok(true) } else { Ok(false) } // Placeholder
     }
     fn create_backup(&self) -> io::Result<Vec<PathBuf>> {
         println!("Warning: kernelstub backup not applicable or implemented.");
         // kernelstub manages its own state/backups internally?
         Ok(Vec::new())
     }
     fn update_bootloader(&self, _dry_run: bool) -> io::Result<()> {
         // kernelstub applies changes immediately, no separate update needed
         println!("kernelstub configuration updated.");
         Ok(())
     }
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