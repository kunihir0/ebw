// src/core/bootloader/grub.rs

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

use super::BootloaderManager; // Import the trait from the parent module
use crate::utils::create_timestamped_backup; // Import backup utility

/// GRUB bootloader configuration manager
#[derive(Debug)] // Added Debug derive
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
        // Using (?m) flag for multiline matching
        let re = Regex::new(r#"(?m)^\s*GRUB_CMDLINE_LINUX_DEFAULT\s*=\s*(["'])(.*?)\1"#)
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
                 return Ok(()); // Don't error out, just warn
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