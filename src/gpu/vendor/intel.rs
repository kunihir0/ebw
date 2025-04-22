// Intel GPU vendor-specific handling
//
// This module implements Intel-specific GPU passthrough handling
// It supports both integrated GPUs with GVT-g and discrete Arc GPUs

use crate::gpu::GpuDevice;
use crate::gpu::GpuVendor;
use crate::gpu::vendor::{GpuVendorHandler, QuirkSetting};
use std::path::Path;
use std::fs;

/// Handler for Intel GPUs
pub struct IntelGpuHandler;

impl GpuVendorHandler for IntelGpuHandler {
    fn name(&self) -> &'static str {
        "Intel GPU Handler"
    }
    
    fn supports_device(&self, device: &GpuDevice) -> bool {
        matches!(device.vendor, GpuVendor::Intel)
    }
    
    fn prepare_for_passthrough(&self, device: &GpuDevice) -> Result<(), String> {
        if device.is_integrated {
            // Check for GVT-g capability
            if device.capabilities.supports_gvt {
                println!("Intel integrated GPU may support GVT-g virtualization.");
                println!("Consider using GVT-g for sharing the GPU with the host.");
                println!("Check if i915.enable_gvt=1 is set in kernel parameters.");
                
                // Check if GVT-g is enabled in kernel
                let gvt_enabled = check_gvt_enabled(device);
                if !gvt_enabled {
                    println!("GVT-g does not appear to be enabled. Add i915.enable_gvt=1 to kernel parameters.");
                }
            } else {
                println!("Warning: Full passthrough of Intel integrated GPU may not work well.");
                println!("Passthrough of Intel integrated GPU may leave host without display.");
            }
        } else {
            // Arc or other discrete GPU
            if device.model_name.to_lowercase().contains("arc") {
                println!("Intel Arc GPU detected.");
                println!("Arc GPUs are relatively new - passthrough support may vary.");
            }
        }
        
        Ok(())
    }
    
    fn apply_quirks(&self, device: &GpuDevice) -> Result<Vec<QuirkSetting>, String> {
        let mut quirks = Vec::new();
        
        if device.is_integrated && device.capabilities.supports_gvt {
            // GVT-g virtualization for integrated GPUs
            quirks.push(QuirkSetting {
                name: "gvt_g_setup".to_string(),
                description: "Configure GVT-g for Intel integrated GPU".to_string(),
                xml_snippet: Some(r#"<devices>
  <hostdev mode='subsystem' type='mdev' managed='no' model='vfio-pci'>
    <source>
      <address uuid='REPLACE_WITH_MDEV_UUID'/>
    </source>
  </hostdev>
</devices>"#.to_string()),
                command_line_option: None,
            });
            
            // Kernel parameters
            quirks.push(QuirkSetting {
                name: "gvt_kernel_params".to_string(),
                description: "Required kernel parameters for GVT-g".to_string(),
                xml_snippet: None,
                command_line_option: Some("i915.enable_gvt=1 intel_iommu=on".to_string()),
            });
        } else if !device.is_integrated {
            // Arc or other discrete GPU
            quirks.push(QuirkSetting {
                name: "rom_loading".to_string(),
                description: "Load GPU ROM for better compatibility".to_string(),
                xml_snippet: Some(r#"<hostdev>
  <rom bar='on'/>
</hostdev>"#.to_string()),
                command_line_option: None,
            });
            
            // Special considerations for Arc GPUs
            if device.model_name.to_lowercase().contains("arc") {
                quirks.push(QuirkSetting {
                    name: "arc_driver_isolation".to_string(),
                    description: "Ensure proper driver isolation for Arc GPUs".to_string(),
                    xml_snippet: None,
                    command_line_option: None,
                });
            }
        }
        
        Ok(quirks)
    }
    
    fn verify_passthrough_ready(&self, device: &GpuDevice) -> Result<bool, String> {
        // Different checks for integrated vs discrete
        if device.is_integrated && device.capabilities.supports_gvt {
            // Check if GVT-g is properly set up
            // Fix E0716 error by creating a string before creating a Path
            let path_str = format!("/sys/bus/pci/devices/{}/mdev_supported_types", 
                                  device.bdf.replace(':', "/"));
            let gvt_path = Path::new(&path_str);
                                              
            if !gvt_path.exists() {
                return Err("GVT-g support not available - check kernel parameters".to_string());
            }
            
            // Additional checks could be performed here
            return Ok(true);
        } else {
            // Discrete GPU - check if bound to vfio-pci
            if let Some(driver) = &device.driver {
                if driver != "vfio-pci" {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
            
            return Ok(true);
        }
    }
}

/// Check if GVT-g is enabled for an Intel integrated GPU
fn check_gvt_enabled(device: &GpuDevice) -> bool {
    // Check the i915 kernel module parameters
    if let Ok(params) = fs::read_to_string("/sys/module/i915/parameters/enable_gvt") {
        return params.trim() == "Y" || params.trim() == "1";
    }
    
    // Alternative check: look for mdev_supported_types directory
    // Fix E0716 error by creating a string before creating a Path
    let path_str = format!("/sys/bus/pci/devices/{}/mdev_supported_types", 
                          device.bdf.replace(':', "/"));
    let gvt_path = Path::new(&path_str);
    
    gvt_path.exists()
}

/// Create a GVT-g virtual device for an Intel integrated GPU
pub fn create_gvtg_device(device: &GpuDevice, _vm_name: &str, _mem_size_mb: u64) -> Result<String, String> {
    // This is a simplified version - real implementation would:
    // 1. Find appropriate mdev_supported_types
    // 2. Select best size based on mem_size_mb
    // 3. Create UUID
    // 4. Write to create file
    
    // Fix E0716 error by creating a string before creating a Path
    let path_str = format!("/sys/bus/pci/devices/{}/mdev_supported_types", 
                          device.bdf.replace(':', "/"));
    let gvt_path = Path::new(&path_str);
    
    if !gvt_path.exists() {
        return Err("GVT-g support not available".to_string());
    }
    
    // In a real implementation, we'd:
    // 1. Check subdirectories to find appropriate device types
    // 2. Read available_instances to check capacity
    // 3. Generate UUID and write to create file
    
    Err("GVT-g device creation not fully implemented".to_string())
}