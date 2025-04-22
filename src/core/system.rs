// System detection module for Exliar VFIO Automation Framework
//
// This module handles detection of system properties such as:
// - Bootloader type (GRUB, systemd-boot, etc.)
// - Kernel version information
// - CPU vendor and features
// - Init system (systemd, OpenRC, etc.)
// - Distribution details

use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents the bootloader type detected on the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootloaderType {
    Grub,
    SystemdBoot,
    PopOsKernelstub,
    Other(String),
    Unknown,
}

/// Represents the system's CPU vendor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuVendor {
    AMD,
    Intel,
    Other(String),
}

/// Represents the system's init system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitSystem {
    Systemd,
    OpenRC,
    SysVInit,
    Other(String),
    Unknown,
}

/// Represents the system's initramfs system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitramfsSystem {
    Dracut,
    Mkinitcpio,
    Booster,
    Debian,
    Other(String),
    Unknown,
}

/// Represents the base distribution family
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistroFamily { // Make enum public
    Arch,
    Debian,
    Fedora,
    Suse,
    Gentoo,
    Other(String),
}


/// Holds information about the Linux distribution
#[derive(Debug, Clone)]
pub struct Distribution { // Make struct public
    pub name: String,
    pub version: String,
    pub id: String,
    pub family: Option<DistroFamily>, // Ensure field is public (it is by default within pub struct)
}

/// Holds information about the kernel version
#[derive(Debug, Clone)]
pub struct KernelVersion { // Make struct public
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
    pub full_version: String,
}

/// Contains all detected system information
#[derive(Debug, Clone)] // Add Clone derive
pub struct SystemInfo { // Make struct public
    pub bootloader: BootloaderType,
    pub kernel_version: KernelVersion,
    pub cpu_vendor: CpuVendor,
    pub virtualization_enabled: bool,
    pub init_system: InitSystem,
    pub initramfs_system: InitramfsSystem,
    pub secure_boot_enabled: Option<bool>,
    pub distribution: Option<Distribution>,
}

impl SystemInfo {
    /// Detects and collects all system information
    pub fn detect() -> Self {
        SystemInfo {
            bootloader: detect_bootloader(),
            kernel_version: detect_kernel_version(),
            cpu_vendor: detect_cpu_vendor(),
            virtualization_enabled: check_virtualization_support(),
            init_system: detect_init_system(),
            initramfs_system: detect_initramfs_system(),
            secure_boot_enabled: detect_secure_boot(),
            distribution: detect_distribution(),
        }
    }

    /// Returns a textual summary of the system information
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Kernel: {}\n", self.kernel_version.full_version));
        summary.push_str(&format!("Bootloader: {:?}\n", self.bootloader));
        summary.push_str(&format!("CPU Vendor: {:?}\n", self.cpu_vendor));
        summary.push_str(&format!("Virtualization: {}\n", 
            if self.virtualization_enabled { "Enabled" } else { "Disabled" }));
        
        if let Some(ref distro) = self.distribution {
             let family_str = match &distro.family {
                 Some(family) => format!(" ({:?}-based)", family),
                 None => String::new(),
             };
            summary.push_str(&format!("Distribution: {} {}{}\n", distro.name, distro.version, family_str));
        }
        
        summary.push_str(&format!("Init System: {:?}\n", self.init_system));
        summary.push_str(&format!("Initramfs System: {:?}\n", self.initramfs_system));
        
        if let Some(secure_boot) = self.secure_boot_enabled {
            summary.push_str(&format!("Secure Boot: {}\n", 
                if secure_boot { "Enabled" } else { "Disabled" }));
        } else {
            summary.push_str("Secure Boot: Unknown\n");
        }
        
        summary
    }
}

/// Detects the bootloader type used by the system
fn detect_bootloader() -> BootloaderType {
    // Check for GRUB
    if Path::new("/etc/default/grub").exists() {
        return BootloaderType::Grub;
    }
    
    // Check for systemd-boot
    if Path::new("/boot/efi/loader/loader.conf").exists() || Path::new("/boot/loader/loader.conf").exists() {
        // Check specifically for Pop!_OS
        let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
        if os_release.contains("ID=pop") {
            return BootloaderType::PopOsKernelstub;
        }
        return BootloaderType::SystemdBoot;
    }
    
    // Unknown bootloader
    BootloaderType::Unknown
}

/// Detects the Linux kernel version
fn detect_kernel_version() -> KernelVersion {
    // Try to read kernel version using `uname -r`
    let output = Command::new("uname")
        .arg("-r")
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            let version_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            parse_kernel_version(&version_str)
        }
        _ => {
            // Fallback to /proc/version
            if let Ok(version) = fs::read_to_string("/proc/version") {
                // Extract version from /proc/version format
                if let Some(ver_str) = version.split_whitespace().nth(2) {
                    return parse_kernel_version(ver_str);
                }
            }
            
            // Default fallback
            KernelVersion {
                major: 0,
                minor: 0,
                patch: None,
                full_version: "unknown".to_string(),
            }
        }
    }
}

/// Parses kernel version string into components
fn parse_kernel_version(version_str: &str) -> KernelVersion {
    // Extract version components with regex-like logic
    let parts: Vec<&str> = version_str.split('.').collect();
    
    let major = parts.get(0)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    
    let minor = parts.get(1)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    
    // Patch might include extra parts like "-generic"
    let patch = parts.get(2)
        .and_then(|v| {
            // Extract numeric part if there's non-numeric suffix
            let num_part: String = v.chars()
                .take_while(|c| c.is_digit(10))
                .collect();
            num_part.parse::<u32>().ok()
        });
    
    KernelVersion {
        major,
        minor,
        patch,
        full_version: version_str.to_string(),
    }
}

/// Detects the CPU vendor (AMD, Intel, etc.)
fn detect_cpu_vendor() -> CpuVendor {
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("vendor_id") {
                if line.contains("AuthenticAMD") {
                    return CpuVendor::AMD;
                } else if line.contains("GenuineIntel") {
                    return CpuVendor::Intel;
                } else if let Some(vendor) = line.split(':').nth(1) {
                    return CpuVendor::Other(vendor.trim().to_string());
                }
                break; // Only need the first occurrence
            }
        }
    }
    
    CpuVendor::Other("Unknown".to_string())
}

/// Checks if CPU virtualization is enabled
fn check_virtualization_support() -> bool {
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        // Check for AMD-V (svm) or Intel VT-x (vmx) flag
        for line in cpuinfo.lines() {
            if line.starts_with("flags") {
                return line.contains(" svm ") || line.contains(" vmx ");
            }
        }
    }
    
    false
}

/// Detects the init system being used
fn detect_init_system() -> InitSystem {
    // Check for systemd
    if Path::new("/run/systemd/system").exists() {
        return InitSystem::Systemd;
    }
    
    // Check for OpenRC
    if Path::new("/run/openrc").exists() || Path::new("/lib/rc/init.d").exists() {
        return InitSystem::OpenRC;
    }
    
    // Check for SysVInit
    if Path::new("/etc/inittab").exists() {
        return InitSystem::SysVInit;
    }
    
    // Unknown init system
    InitSystem::Unknown
}

/// Detects the initramfs system being used
fn detect_initramfs_system() -> InitramfsSystem {
    // Check for mkinitcpio (Arch Linux)
    if Path::new("/etc/mkinitcpio.conf").exists() {
        return InitramfsSystem::Mkinitcpio;
    }
    
    // Check for dracut
    if Path::new("/etc/dracut.conf").exists() || Path::new("/etc/dracut.conf.d").exists() {
        return InitramfsSystem::Dracut;
    }
    
    // Check for booster
    if Path::new("/etc/booster.yaml").exists() || Path::new("/etc/booster.d").exists() {
        return InitramfsSystem::Booster;
    }
    
    // Check for Debian/Ubuntu
    if Path::new("/etc/initramfs-tools").exists() {
        return InitramfsSystem::Debian;
    }
    
    // Unknown initramfs system
    InitramfsSystem::Unknown
}

/// Detects if Secure Boot is enabled
fn detect_secure_boot() -> Option<bool> {
    // Try to use mokutil if available
    let output = Command::new("mokutil")
        .arg("--sb-state")
        .output();
    
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if stdout.contains("secureboot enabled") {
                return Some(true);
            } else if stdout.contains("secureboot disabled") {
                return Some(false);
            }
        }
    }
    
    // Alternative check via EFI variables
    let secure_boot_var = Path::new("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c");
    if secure_boot_var.exists() {
        if let Ok(data) = fs::read(secure_boot_var) {
            if data.len() > 4 {
                // The relevant byte is usually the 5th byte (index 4)
                return Some(data[data.len() - 1] == 1);
            }
        }
    }
    
    // Cannot determine the status
    None
}

/// Detects the Linux distribution and its family
fn detect_distribution() -> Option<Distribution> {
    if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
        let mut name = String::new();
        let mut version = String::new();
        let mut id = String::new();
        let mut id_like = String::new(); // Read ID_LIKE field

        for line in os_release.lines() {
            if line.starts_with("NAME=") {
                name = line.trim_start_matches("NAME=").trim_matches('"').to_string();
            } else if line.starts_with("VERSION=") {
                version = line.trim_start_matches("VERSION=").trim_matches('"').to_string();
            } else if line.starts_with("ID=") {
                id = line.trim_start_matches("ID=").trim_matches('"').to_string();
            } else if line.starts_with("ID_LIKE=") {
                id_like = line.trim_start_matches("ID_LIKE=").trim_matches('"').to_string();
            }
        }

        if !name.is_empty() {
            // Determine family based on ID or ID_LIKE
            let family = determine_distro_family(&id, &id_like);

            return Some(Distribution { name, version, id, family });
        }
    }

    None
}

/// Determines the distribution family based on ID and ID_LIKE fields
fn determine_distro_family(id: &str, id_like: &str) -> Option<DistroFamily> {
    let check_ids: Vec<&str> = id_like.split_whitespace().chain(std::iter::once(id)).collect();

    for check_id in check_ids {
        match check_id {
            "arch" | "garuda" | "manjaro" | "endeavouros" => return Some(DistroFamily::Arch), // Added common Arch derivatives
            "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" => return Some(DistroFamily::Debian),
            "fedora" | "rhel" | "centos" | "rocky" | "alma" => return Some(DistroFamily::Fedora),
            "opensuse" | "suse" | "tumbleweed" | "leap" => return Some(DistroFamily::Suse),
            "gentoo" => return Some(DistroFamily::Gentoo),
            _ => {}
        }
    }
    // If no known family matched, return Other based on the primary ID
    if !id.is_empty() {
         Some(DistroFamily::Other(id.to_string()))
    } else {
         None
    }
}