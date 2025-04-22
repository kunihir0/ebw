// GPU detection module for Exliar VFIO Automation Framework
//
// This module handles detection of GPU devices in the system using
// PCI bus scanning and device identification

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::gpu::{GpuDevice, GpuVendor, GpuDriverCapabilities};

/// Describes a detected PCI device
#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bdf: String,               // Bus:Device.Function address (e.g., "01:00.0")
    pub vendor_id: String,         // Vendor ID (e.g., "10de" for NVIDIA)
    pub device_id: String,         // Device ID
    pub class: String,             // Device class (e.g., "VGA compatible controller")
    pub vendor_name: String,       // Vendor name (e.g., "NVIDIA Corporation")
    pub device_name: String,       // Device name (e.g., "GeForce RTX 3080")
    pub driver: Option<String>,    // Current driver
    pub sysfs_path: PathBuf,       // Path to device in sysfs
}

/// Detects all GPU devices in the system
pub fn detect_gpus() -> Vec<GpuDevice> {
    // First, get all PCI devices
    let pci_devices = get_pci_devices();
    
    // Filter for GPU devices
    let gpu_pci_devices: Vec<_> = pci_devices.iter()
        .filter(|dev| is_gpu_device(dev))
        .collect();
    
    // Convert to GpuDevice objects with additional information
    gpu_pci_devices.iter()
        .map(|dev| pci_to_gpu_device(dev))
        .collect()
}

/// Check if a PCI device is a GPU
fn is_gpu_device(device: &PciDevice) -> bool {
    // VGA compatible controller, Display controller, or 3D controller
    device.class.contains("VGA") || 
    device.class.contains("display controller") ||
    device.class.contains("3D controller")
}

/// Converts a PCI device to a GPU device with additional information
fn pci_to_gpu_device(device: &PciDevice) -> GpuDevice {
    // Determine vendor
    let vendor = detect_gpu_vendor(device);
    
    // Set GPU capabilities based on vendor and device ID
    let capabilities = detect_gpu_capabilities(device, &vendor);
    
    // Determine if integrated
    let is_integrated = is_integrated_gpu(device, &vendor);
    
    // Try to determine VRAM size
    let vram_size = detect_vram_size(device);
    
    GpuDevice {
        bdf: device.bdf.clone(),
        vendor_id: device.vendor_id.clone(),
        device_id: device.device_id.clone(),
        vendor,
        model_name: device.device_name.clone(),
        is_integrated,
        vram_size,
        driver: device.driver.clone(),
        capabilities,
    }
}

/// Determines the vendor based on vendor ID and name
fn detect_gpu_vendor(device: &PciDevice) -> GpuVendor {
    // Check vendor ID first
    match device.vendor_id.as_str() {
        "1002" => GpuVendor::AMD,
        "10de" => GpuVendor::NVIDIA,
        "8086" => GpuVendor::Intel,
        _ => {
            // If vendor ID didn't match, try vendor name
            let vendor_name = device.vendor_name.to_lowercase();
            if vendor_name.contains("amd") || vendor_name.contains("ati") {
                GpuVendor::AMD
            } else if vendor_name.contains("nvidia") {
                GpuVendor::NVIDIA
            } else if vendor_name.contains("intel") {
                GpuVendor::Intel
            } else {
                GpuVendor::Other(device.vendor_name.clone())
            }
        }
    }
}

/// Detects GPU capabilities based on vendor and device ID
fn detect_gpu_capabilities(device: &PciDevice, vendor: &GpuVendor) -> GpuDriverCapabilities {
    let mut capabilities = GpuDriverCapabilities::default();
    
    match vendor {
        GpuVendor::AMD => {
            // AMD reset bug affects many older cards
            // This is a simplified check - we would want more detailed per-device checks
            let device_id = u32::from_str_radix(&device.device_id, 16).unwrap_or(0);
            if device_id < 0x7000 {  // Rough estimation for pre-RDNA cards
                capabilities.has_reset_bug = true;
            }
            
            capabilities.supports_reset = !capabilities.has_reset_bug;
            capabilities.supports_vbios_loading = true;
        },
        GpuVendor::NVIDIA => {
            // NVIDIA GPUs need code 43 workaround for Windows guests
            capabilities.needs_code_43_workaround = true;
            capabilities.supports_reset = true;
            capabilities.supports_vbios_loading = true;
        },
        GpuVendor::Intel => {
            // Intel integrated GPUs often support GVT-g
            if device.device_name.to_lowercase().contains("hd graphics") ||
               device.device_name.to_lowercase().contains("uhd graphics") {
                capabilities.supports_gvt = true;
            }
            // Arc GPUs are discrete
            if device.device_name.to_lowercase().contains("arc") {
                capabilities.supports_reset = true;
                capabilities.supports_vbios_loading = true;
            }
        },
        GpuVendor::Other(_) => {
            // Unknown vendor - assume minimal capabilities
            capabilities.supports_reset = false;
            capabilities.supports_vbios_loading = false;
        }
    }
    
    capabilities
}

/// Determines if a GPU is integrated with the CPU
fn is_integrated_gpu(device: &PciDevice, vendor: &GpuVendor) -> bool {
    match vendor {
        GpuVendor::Intel => {
            // Most Intel GPUs are integrated except for Arc series
            !device.device_name.to_lowercase().contains("arc")
        },
        GpuVendor::AMD => {
            // AMD APUs have integrated graphics
            device.device_name.to_lowercase().contains("radeon") &&
            (device.device_name.to_lowercase().contains("vega") || 
             device.device_name.to_lowercase().contains("graphics"))
        },
        _ => false,
    }
}

/// Attempts to detect GPU VRAM size
fn detect_vram_size(_device: &PciDevice) -> Option<u64> {
    // For now, we'll use a simplified implementation
    // A real implementation would read from sysfs or other sources
    None
}

/// Gets a list of all PCI devices in the system (simplified version)
/// In a real implementation, we'd use a proper PCI library or direct sysfs access
fn get_pci_devices() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    
    // This is a simplified implementation using lspci
    // Ideally, we'd use a library for this or direct sysfs access
    let output = Command::new("lspci")
        .args(["-vmm"])
        .output();
        
    if let Ok(output) = output {
        if output.status.success() {
            // Parse lspci -vmm output (simplified)
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Split output by device (double newline)
            let device_sections: Vec<&str> = stdout.split("\n\n").collect();
            
            for section in device_sections {
                if section.trim().is_empty() {
                    continue;
                }
                
                // Parse device attributes
                let mut bdf = String::new();
                let mut class = String::new();
                let mut vendor_name = String::new();
                let mut device_name = String::new();
                // vendor_id and device_id will be set later
                
                for line in section.lines() {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    
                    match key {
                        "Slot" => bdf = value.to_string(),
                        "Class" => class = value.to_string(),
                        "Vendor" => vendor_name = value.to_string(),
                        "Device" => device_name = value.to_string(),
                        _ => {}
                    }
                }
                
                // We need to get vendor/device IDs separately
                if !bdf.is_empty() {
                    // Initialize vendor_id based on vendor name
                    let vendor_id = if vendor_name.contains("NVIDIA") {
                        "10de".to_string()
                    } else if vendor_name.contains("AMD") || vendor_name.contains("ATI") {
                        "1002".to_string()
                    } else if vendor_name.contains("Intel") {
                        "8086".to_string()
                    } else {
                        "0000".to_string()
                    };
                    
                    // Initialize device_id
                    let device_id = "0000".to_string();
                    
                    // Build the sysfs path
                    let sysfs_path = PathBuf::from(format!("/sys/bus/pci/devices/{}", bdf.replace(':', "/")));
                    
                    // Check if the device has a driver
                    let driver = get_device_driver(&sysfs_path);
                    
                    devices.push(PciDevice {
                        bdf,
                        vendor_id,
                        device_id,
                        class,
                        vendor_name,
                        device_name,
                        driver,
                        sysfs_path,
                    });
                }
            }
        }
    }
    
    devices
}

/// Gets the current driver for a PCI device from sysfs
fn get_device_driver(sysfs_path: &Path) -> Option<String> {
    let driver_path = sysfs_path.join("driver");
    if driver_path.exists() {
        if let Ok(driver) = fs::read_link(driver_path) {
            return driver.file_name()
                .map(|name| name.to_string_lossy().to_string());
        }
    }
    None
}