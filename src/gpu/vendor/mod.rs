// GPU vendor-specific handling module
//
// This module contains vendor-specific implementations for GPU passthrough
// handling across different vendors (AMD, NVIDIA, Intel)

pub mod amd;
pub mod nvidia;
pub mod intel;

use crate::gpu::GpuDevice;

/// Trait for GPU vendor-specific operations
pub trait GpuVendorHandler {
    /// Name of the handler
    fn name(&self) -> &'static str;
    
    /// Check if this handler supports the given GPU
    fn supports_device(&self, device: &GpuDevice) -> bool;
    
    /// Prepares a GPU for passthrough (unbind, configure, etc.)
    fn prepare_for_passthrough(&self, device: &GpuDevice) -> Result<(), String>;
    
    /// Apply vendor-specific quirks for VM configuration
    fn apply_quirks(&self, device: &GpuDevice) -> Result<Vec<QuirkSetting>, String>;
    
    /// Perform post-bind checks
    fn verify_passthrough_ready(&self, device: &GpuDevice) -> Result<bool, String>;
}

/// Represents a quirk setting for VM configuration
#[derive(Debug, Clone)]
pub struct QuirkSetting {
    pub name: String,
    pub description: String,
    pub xml_snippet: Option<String>,
    pub command_line_option: Option<String>,
}