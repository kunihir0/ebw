// VFIO Module Management for Exliar VFIO Automation Framework
//
// This module handles configuring the system to use VFIO drivers
// for specified PCI devices, including modprobe configuration,
// initramfs updates, and device binding.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command; // Import Command

use crate::core::system::{SystemInfo, InitramfsSystem}; // Import InitramfsSystem
use crate::gpu::detection::PciDevice;

/// Manages VFIO configuration and device binding
pub struct VfioManager {
    system_info: SystemInfo,
    // Potentially add configuration options here
}

impl VfioManager {
    /// Creates a new VfioManager
    pub fn new(system_info: SystemInfo) -> Self {
        Self { system_info }
    }

    /// Configures /etc/modprobe.d/ for VFIO modules
    ///
    /// Args:
    ///     device_ids: List of vendor:device ID strings (e.g., "10de:1eb1")
    ///     dry_run: If true, only log actions without modifying files
    ///
    /// Returns:
    ///     Result indicating success or failure
    pub fn configure_modprobe(&self, device_ids: &[String], dry_run: bool) -> io::Result<()> {
        println!("Configuring VFIO driver options via modprobe...");

        let modprobe_dir = Path::new("/etc/modprobe.d");
        let vfio_conf_path = modprobe_dir.join("vfio.conf");

        // Ensure target directory exists
        if !dry_run && !modprobe_dir.exists() {
            println!("Creating directory {}...", modprobe_dir.display());
            fs::create_dir_all(modprobe_dir)?;
        }

        // Prepare configuration lines
        let ids_string = device_ids.join(",");
        // disable_vga=1 prevents vfio-pci from binding to the primary device if it's VGA
        // disable_idle_d3=1 recommended for stability with some devices
        let options_line = format!("options vfio-pci ids={} disable_vga=1 disable_idle_d3=1", ids_string);

        // Soft dependencies to ensure vfio-pci loads before graphics drivers
        let softdep_lines = [
            "softdep drm pre: vfio-pci",
            "softdep amdgpu pre: vfio-pci",
            "softdep nouveau pre: vfio-pci",
            "softdep radeon pre: vfio-pci",
            "softdep nvidia pre: vfio-pci",
            "softdep i915 pre: vfio-pci",
        ];

        if dry_run {
            println!("[DRY RUN] Would write/update {}:", vfio_conf_path.display());
            println!("  {}", options_line);
            for line in softdep_lines {
                println!("  {}", line);
            }
            // Also ensure modules load early via /etc/modules-load.d/
            let modules_load_path = Path::new("/etc/modules-load.d/vfio-pci-load.conf");
            println!("[DRY RUN] Would ensure VFIO modules are listed in {}", modules_load_path.display());
            return Ok(());
        }

        // --- Configure vfio.conf ---
        let current_vfio_content = fs::read_to_string(&vfio_conf_path).unwrap_or_default();
        let mut new_vfio_lines = Vec::new();
        let mut vfio_pci_option_found = false;
        let mut existing_softdeps = std::collections::HashSet::new();

        for line in current_vfio_content.lines() {
            let stripped_line = line.trim();
            if stripped_line.is_empty() || stripped_line.starts_with('#') {
                new_vfio_lines.push(line.to_string());
                continue;
            }
            if stripped_line.starts_with("options vfio-pci") {
                if !vfio_pci_option_found {
                    new_vfio_lines.push(options_line.clone());
                    vfio_pci_option_found = true;
                    println!("Replacing existing options vfio-pci line.");
                } else {
                    new_vfio_lines.push(format!("# {}", line));
                    println!("Commenting out duplicate options vfio-pci line.");
                }
            } else if stripped_line.starts_with("softdep ") && stripped_line.contains(" pre: vfio-pci") {
                existing_softdeps.insert(stripped_line.to_string());
                new_vfio_lines.push(line.to_string());
            } else {
                new_vfio_lines.push(line.to_string());
            }
        }
        if !vfio_pci_option_found {
            new_vfio_lines.push(options_line);
            println!("Adding new options vfio-pci line.");
        }
        for softdep in softdep_lines {
            if !existing_softdeps.contains(softdep) {
                new_vfio_lines.push(softdep.to_string());
                println!("Adding missing softdep line: {}", softdep);
            }
        }
        let new_vfio_content_str = new_vfio_lines.join("\n") + "\n";
        if new_vfio_content_str != current_vfio_content {
            let backup_path = create_timestamped_backup(&vfio_conf_path)?;
            println!("Created backup: {}", backup_path.display());
            let mut file = fs::File::create(&vfio_conf_path)?;
            file.write_all(new_vfio_content_str.as_bytes())?;
            println!("Successfully updated {}", vfio_conf_path.display());
        } else {
            println!("{} is already up-to-date.", vfio_conf_path.display());
        }

        // --- Configure modules-load.d ---
        let modules_load_dir = Path::new("/etc/modules-load.d");
        if !modules_load_dir.exists() {
             println!("Creating directory {}...", modules_load_dir.display());
             fs::create_dir_all(modules_load_dir)?;
        }
        let modules_load_path = modules_load_dir.join("vfio-pci-load.conf");
        let vfio_modules_to_load = [
            "vfio",
            "vfio_iommu_type1",
            "vfio_pci",
            // vfio_virqfd is often needed for older kernels, include for broader compatibility
            "vfio_virqfd",
        ];
        let modules_load_content = vfio_modules_to_load.join("\n") + "\n";
        let current_load_content = fs::read_to_string(&modules_load_path).unwrap_or_default();

        if modules_load_content != current_load_content {
             let backup_path = create_timestamped_backup(&modules_load_path)?;
             println!("Created backup: {}", backup_path.display());
             let mut file = fs::File::create(&modules_load_path)?;
             file.write_all(modules_load_content.as_bytes())?;
             println!("Successfully updated {}", modules_load_path.display());
        } else {
             println!("{} is already up-to-date.", modules_load_path.display());
        }

        Ok(())
    }

    /// Updates the initramfs based on the detected system type
    ///
    /// Args:
    ///     dry_run: If true, only log actions without modifying files
    ///
    /// Returns:
    ///     Result indicating success or failure
    pub fn update_initramfs(&self, dry_run: bool) -> io::Result<()> {
        println!("Updating initramfs using {:?}...", self.system_info.initramfs_system);

        // Determine the command based on the detected initramfs system
        let command_parts = match self.system_info.initramfs_system {
            InitramfsSystem::Mkinitcpio => Some(vec!["mkinitcpio", "-P"]),
            InitramfsSystem::Dracut => {
                // Dracut might need specific arguments depending on distro (e.g., Arch)
                // A more robust implementation would check distro family here
                Some(vec!["dracut", "--force"])
            },
            InitramfsSystem::Debian => Some(vec!["update-initramfs", "-u", "-k", "all"]),
            InitramfsSystem::Booster => Some(vec!["booster", "build"]), // Assuming booster command
            _ => {
                println!("Warning: Unsupported or unknown initramfs system ({:?}). Cannot update automatically.", self.system_info.initramfs_system);
                return Ok(()); // Not an error, just can't proceed
            }
        };

        if let Some(parts) = command_parts {
            let command_str = parts.join(" ");
            if dry_run {
                println!("[DRY RUN] Would execute: {}", command_str);
                return Ok(());
            }

            println!("Executing: {}", command_str);
            let status = Command::new(parts[0])
                .args(&parts[1..])
                .status()?; // Use status() to wait for completion and get exit code

            if status.success() {
                println!("Initramfs updated successfully.");
                Ok(())
            } else {
                let err_msg = format!("Initramfs update command failed with exit code: {:?}", status.code());
                println!("Error: {}", err_msg);
                Err(io::Error::new(io::ErrorKind::Other, err_msg))
            }
        } else {
             // This case should ideally not be reached due to the match statement covering '_'
             Ok(())
        }
    }

    /// Binds a specific PCI device to the vfio-pci driver
    ///
    /// Args:
    ///     device: The PCI device to bind
    ///     dry_run: If true, only log actions without modifying files
    ///
    /// Returns:
    ///     Result indicating success or failure
    pub fn bind_device(&self, device: &PciDevice, dry_run: bool) -> io::Result<()> {
        println!("Binding device {} to vfio-pci...", device.bdf);

        let device_sysfs_path = &device.sysfs_path;
        let driver_override_path = device_sysfs_path.join("driver_override");
        let unbind_path = device_sysfs_path.join("driver/unbind");
        let bind_path = Path::new("/sys/bus/pci/drivers/vfio-pci/bind");

        if dry_run {
            println!("[DRY RUN] Would perform the following steps:");
            if let Some(driver) = &device.driver {
                 println!("  - Unbind from current driver '{}' at {}", driver, unbind_path.display());
            }
            println!("  - Set driver_override to 'vfio-pci' at {}", driver_override_path.display());
            println!("  - Bind to vfio-pci driver at {}", bind_path.display());
            return Ok(());
        }

        // 1. Unbind from current driver if necessary
        if let Some(_driver) = &device.driver {
            if unbind_path.exists() {
                println!("  Unbinding from current driver...");
                // Use write! macro for better error handling potential
                let mut unbind_file = fs::OpenOptions::new().write(true).open(&unbind_path)?;
                write!(unbind_file, "{}", device.bdf)?;
                // Small delay to allow unbinding to complete
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                 println!("  No 'unbind' path found for current driver, skipping unbind.");
            }
        }

        // 2. Set driver_override to vfio-pci
        println!("  Setting driver_override to vfio-pci...");
        // Ensure the file exists before writing, handle potential errors
        if driver_override_path.exists() {
            let mut override_file = fs::OpenOptions::new().write(true).truncate(true).open(&driver_override_path)?;
            write!(override_file, "vfio-pci")?;
        } else {
             println!("  driver_override path not found, skipping set.");
             // Depending on the system, this might be okay or an error.
             // For now, we'll just warn. A more robust implementation might error here.
        }


        // 3. Bind to vfio-pci driver
        println!("  Binding to vfio-pci driver...");
        if bind_path.exists() {
             let mut bind_file = fs::OpenOptions::new().write(true).open(&bind_path)?;
             write!(bind_file, "{}", device.bdf)?;
        } else {
             return Err(io::Error::new(io::ErrorKind::NotFound, "vfio-pci bind path not found. Is vfio-pci module loaded?"));
        }

        println!("Device {} successfully bound to vfio-pci.", device.bdf);
        Ok(())
    }

     /// Unbinds a specific PCI device from the vfio-pci driver
    ///
    /// Args:
    ///     device: The PCI device to unbind
    ///     dry_run: If true, only log actions without modifying files
    ///
    /// Returns:
    ///     Result indicating success or failure
    pub fn unbind_device(&self, device: &PciDevice, dry_run: bool) -> io::Result<()> {
        println!("Unbinding device {} from vfio-pci...", device.bdf);

        let device_sysfs_path = &device.sysfs_path;
        let vfio_unbind_path = Path::new("/sys/bus/pci/drivers/vfio-pci/unbind");
        let driver_override_path = device_sysfs_path.join("driver_override");
        let probe_path = Path::new("/sys/bus/pci/drivers_probe");

        if dry_run {
            println!("[DRY RUN] Would perform the following steps:");
            println!("  - Unbind from vfio-pci driver at {}", vfio_unbind_path.display());
            println!("  - Clear driver_override at {}", driver_override_path.display());
            println!("  - Trigger re-probe using {}", probe_path.display());
            return Ok(());
        }

        // 1. Unbind from vfio-pci
        println!("  Unbinding from vfio-pci...");
        if vfio_unbind_path.exists() {
            let mut unbind_file = fs::OpenOptions::new().write(true).open(&vfio_unbind_path)?;
            write!(unbind_file, "{}", device.bdf)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            // If vfio-pci isn't loaded or device isn't bound, this might not exist.
            // Consider if this should be a warning or an error.
             println!("  vfio-pci unbind path not found, device might not be bound.");
             // Let's proceed to clear override and reprobe anyway.
        }

        // 2. Clear driver_override
        println!("  Clearing driver_override...");
        if driver_override_path.exists() {
             let mut override_file = fs::OpenOptions::new().write(true).truncate(true).open(&driver_override_path)?;
             write!(override_file, "")?; // Write empty string to clear
        } else {
             println!("  driver_override path not found, skipping clear.");
        }

        // 3. Trigger device re-probe to bind back to original driver
        println!("  Triggering device re-probe...");
        if probe_path.exists() {
             let mut probe_file = fs::OpenOptions::new().write(true).open(&probe_path)?;
             write!(probe_file, "{}", device.bdf)?;
        } else {
             // This path should generally exist.
             return Err(io::Error::new(io::ErrorKind::NotFound, "drivers_probe path not found"));
        }

        println!("Device {} successfully unbound and re-probed.", device.bdf);
        Ok(())
    }
}

/// Helper to create a timestamped backup of a file
fn create_timestamped_backup(file_path: &Path) -> io::Result<PathBuf> {
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