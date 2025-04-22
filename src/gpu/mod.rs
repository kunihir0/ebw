// GPU management module for Exliar VFIO Automation Framework
//
// This module handles GPU device detection, identification, and
// vendor-specific handling for VFIO passthrough

pub mod detection;
pub mod vendor;

use std::fmt;

/// GPU vendors supported by the framework
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    AMD,
    NVIDIA,
    Intel,
    Other(String),
}

// Implement Display for GpuVendor
impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GpuVendor::AMD => write!(f, "AMD"),
            GpuVendor::NVIDIA => write!(f, "NVIDIA"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Other(name) => write!(f, "{}", name),
        }
    }
}

/// Information about GPU driver capabilities
#[derive(Debug, Clone, Default)]
pub struct GpuDriverCapabilities {
    pub supports_reset: bool,      // Can GPU be reset without system reboot
    pub requires_acs_override: bool, // Needs ACS override patch for IOMMU separation
    pub supports_fbc: bool,        // Supports frame buffer compression
    pub supports_gvt: bool,        // Supports Intel GVT-g virtualization
    pub has_reset_bug: bool,       // Has the AMD reset bug
    pub needs_code_43_workaround: bool, // Needs NVIDIA Code 43 workaround
    pub supports_vbios_loading: bool, // Supports custom VBIOS loading
}

/// Represents a detected GPU device
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub bdf: String,               // Bus:Device.Function address (e.g., "01:00.0")
    pub vendor_id: String,         // Vendor ID (e.g., "10de" for NVIDIA)
    pub device_id: String,         // Device ID
    pub vendor: GpuVendor,         // Vendor enum
    pub model_name: String,        // GPU model name
    pub is_integrated: bool,       // Whether the GPU is integrated with CPU
    pub vram_size: Option<u64>,    // VRAM size in MB (if detected)
    pub driver: Option<String>,    // Current driver in use
    pub capabilities: GpuDriverCapabilities,
}

impl GpuDevice {
    /// Returns the BDF (Bus:Device.Function) address
    pub fn bdf(&self) -> &str {
        &self.bdf
    }

    /// Returns the vendor 
    pub fn vendor(&self) -> &GpuVendor {
        &self.vendor
    }
    
    /// Returns the model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}