// Input handling for the TUI

use crossterm::event::{KeyCode, KeyModifiers};
use super::state::{AppState, LogLevel};
use super::app::{detect_system_info, detect_and_log_gpus}; // Import the functions
// Import Change enum for state tracking
use crate::core::state::Change; 

/// Handles key events for the application
pub fn handle_key_event(app: &mut AppState, key_code: KeyCode, modifiers: KeyModifiers) {
    match key_code {
        KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if app.show_gpu_details {
                app.show_gpu_details = false;
                app.add_log("Returning to main dashboard view", LogLevel::Info);
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Up => {
            if app.show_gpu_details && app.gpus.as_ref().map_or(0, |g| g.len()) > 0 {
                if app.selected_gpu_index > 0 {
                    app.selected_gpu_index -= 1;
                } else {
                    // Wrap around to the last GPU
                    app.selected_gpu_index = app.gpus.as_ref().unwrap().len() - 1;
                }
            }
        }
        KeyCode::Down => {
            if app.show_gpu_details {
                if let Some(gpus) = &app.gpus {
                    if app.selected_gpu_index < gpus.len().saturating_sub(1) {
                        app.selected_gpu_index += 1;
                    } else {
                        // Wrap around to the first GPU
                        app.selected_gpu_index = 0;
                    }
                }
            }
        }
        KeyCode::Char('g') => {
            if app.gpus.as_ref().map_or(0, |g| g.len()) > 0 {
                app.show_gpu_details = !app.show_gpu_details;
                app.add_log(if app.show_gpu_details {
                    "Showing detailed GPU information (Use ↑/↓ to navigate, 's' to select, Esc to go back)"
                } else {
                    "Returning to main dashboard view"
                }, LogLevel::Info);
            } else {
                app.add_log("No GPUs detected to show details for.", LogLevel::Warning);
            }
        }
        KeyCode::Char('s') => { // Select GPU for passthrough
            if app.show_gpu_details {
                if let Some(gpus) = &app.gpus {
                    if app.selected_gpu_index < gpus.len() {
                        let selected_gpu = &gpus[app.selected_gpu_index];
                        app.selected_passthrough_gpu_index = Some(app.selected_gpu_index);
                        app.add_log(&format!("Selected GPU {} ({}) for passthrough.",
                                             selected_gpu.bdf(), selected_gpu.model_name()),
                                    LogLevel::Success);
                        app.show_gpu_details = false; // Go back to dashboard after selection
                    }
                }
            }
        }
        KeyCode::Char('c') => { // Configure system for selected GPU
            if let Some(gpu_index) = app.selected_passthrough_gpu_index {
                // --- Prepare Data (Immutable Borrows OK) ---
                let gpu_bdf = app.gpus.as_ref().and_then(|g| g.get(gpu_index)).map(|gpu| gpu.bdf.clone());
                let gpu_model = app.gpus.as_ref().and_then(|g| g.get(gpu_index)).map(|gpu| gpu.model_name.clone());
                // TODO: Get *all* related device IDs for the selected GPU, not just the main one
                let gpu_ids = app.gpus.as_ref().and_then(|g| g.get(gpu_index)).map(|gpu| vec![format!("{}:{}", gpu.vendor_id, gpu.device_id)]);
                let bootloader_name = app.system_info.as_ref().map(|si| format!("{:?}", si.bootloader)); // Get bootloader name for logging/state

                if let (Some(bdf), Some(model), Some(ids), Some(boot_name)) = (gpu_bdf, gpu_model, gpu_ids, bootloader_name) {
                    app.add_log(&format!("Starting configuration for GPU {} ({})", bdf, model), LogLevel::Info);
                    app.current_action = Some(format!("Configuring for {}", model));

                    // --- Perform Actions (Mutable Borrows Separated) ---
                    let mut config_results: Vec<Result<(), String>> = Vec::new();
                    let mut bootloader_updated = false;
                    let mut initramfs_updated = false;
                    let mut changes_to_record: Vec<Change> = Vec::new(); // Buffer changes

                    // 1. Configure Modprobe
                    if let Some(vfio_manager) = &app.vfio_manager {
                        config_results.push(
                            vfio_manager.configure_modprobe(&ids, false)
                                .map_err(|e| format!("Modprobe config failed: {}", e))
                                .map(|_| {
                                    // TODO: Determine actual file paths modified for accurate state tracking
                                    // Example: Assuming vfio_conf_path and backup path are known/returned
                                    // let vfio_conf_path = std::path::PathBuf::from("/etc/modprobe.d/vfio.conf");
                                    // let modules_load_path = std::path::PathBuf::from("/etc/modules-load.d/vfio-pci-load.conf");
                                    // // Need a way to get backup paths if created
                                    // changes_to_record.push(Change::FileModified { path: vfio_conf_path, backup_path: PathBuf::new() });
                                    // changes_to_record.push(Change::FileModified { path: modules_load_path, backup_path: PathBuf::new() });
                                })
                        );
                    } else {
                         config_results.push(Err("VFIO Manager not initialized.".to_string()));
                    }

                    // 2. Add Kernel Parameters & Update Bootloader (if needed and previous steps ok)
                    if config_results.last().map_or(false, |r| r.is_ok()) {
                        if let Some(boot_manager) = app.bootloader_manager.as_mut() {
                            // TODO: Check if params already exist before adding
                            // TODO: Determine correct params based on CPU vendor (from app.system_info)
                            let required_params = ["intel_iommu=on", "iommu=pt"]; // Example
                            match boot_manager.add_parameters(&required_params, false) {
                                Ok(params_changed) => {
                                    if params_changed {
                                        // Record change
                                        for param in required_params {
                                             changes_to_record.push(Change::KernelParamAdded {
                                                 parameter: param.to_string(),
                                                 bootloader: boot_name.clone(),
                                             });
                                        }
                                        // Update Bootloader only if params changed
                                        match boot_manager.update_bootloader(false) {
                                            Ok(_) => {
                                                bootloader_updated = true;
                                                config_results.push(Ok(())); // Step succeeded
                                            },
                                            Err(e) => config_results.push(Err(format!("Bootloader update failed: {}", e))),
                                        }
                                    } else {
                                        config_results.push(Ok(())); // Step succeeded (no change needed)
                                    }
                                },
                                Err(e) => config_results.push(Err(format!("Kernel param add failed: {}", e))),
                            }
                        } else {
                            config_results.push(Err("Bootloader Manager not initialized.".to_string()));
                        }
                    }

                    // 3. Update Initramfs (if previous steps ok)
                    if config_results.last().map_or(false, |r| r.is_ok()) {
                        if let Some(vfio_manager) = &app.vfio_manager {
                            config_results.push(
                                vfio_manager.update_initramfs(false)
                                    .map_err(|e| format!("Initramfs update failed: {}", e))
                                    .map(|_| { initramfs_updated = true; }) // Mark initramfs as updated
                            );
                        }
                        // No else needed for vfio_manager check here as it was checked before
                    }

                    // --- Log Results and Update State (Now safe to borrow app mutably) ---
                    app.current_action = None;
                    let mut overall_success = true;
                    let mut log_buffer: Vec<(String, LogLevel)> = Vec::new();

                    // Process results and buffer logs
                    for result in config_results {
                        match result {
                            Ok(_) => { /* Success logs are handled within functions or below */ }
                            Err(e) => {
                                log_buffer.push((e, LogLevel::Error));
                                overall_success = false;
                            }
                        }
                    }
                     // Add specific success logs based on actions taken
                    if overall_success {
                        log_buffer.push(("Modprobe configured successfully.".to_string(), LogLevel::Success));
                        if bootloader_updated {
                             log_buffer.push(("Kernel parameters added/updated.".to_string(), LogLevel::Success));
                             log_buffer.push(("Bootloader updated successfully.".to_string(), LogLevel::Success));
                        } else {
                             log_buffer.push(("Kernel parameters already set.".to_string(), LogLevel::Info));
                        }
                        if initramfs_updated {
                             log_buffer.push(("Initramfs updated successfully.".to_string(), LogLevel::Success));
                        }
                    }


                    // Add buffered logs to the main app state
                    for (msg, level) in log_buffer {
                        app.add_log(&msg, level);
                    }

                    // Record all buffered changes if successful so far
                    if overall_success {
                         if let Some(state_tracker) = app.state_tracker.as_mut() {
                             for change in changes_to_record {
                                 if let Err(e) = state_tracker.record_change(change) {
                                     app.add_log(&format!("Failed to record state change: {}", e), LogLevel::Error);
                                     overall_success = false; // Mark failure if state saving fails
                                     break; // Stop recording further changes
                                 }
                             }
                         } else {
                             app.add_log("State tracker not available, cannot record changes.", LogLevel::Warning);
                             // Decide if this should be considered a failure? Maybe not critical.
                         }
                    }


                    if overall_success {
                        app.add_log("Initial configuration applied successfully.", LogLevel::Success);
                        app.configuration_applied = true;
                        if bootloader_updated || initramfs_updated {
                            app.reboot_required = true;
                            app.add_log("Reboot required to apply all changes.", LogLevel::Warning);
                        }
                    } else {
                        app.add_log("Configuration failed. See errors above.", LogLevel::Error);
                        // Consider offering rollback?
                        // if let Some(tracker) = app.state_tracker.as_mut() { ... }
                    }
                } else {
                    app.add_log("Selected GPU data is missing.", LogLevel::Error);
                }
            } else {
                app.add_log("No GPU selected for passthrough. Press 'g', select a GPU, then 's'.", LogLevel::Warning);
            }
        }
        KeyCode::Char('r') => {
            app.add_log("Refreshing system information...", LogLevel::Info);
            // Call the functions correctly as free functions, passing the mutable app state
            detect_system_info(app);
            // Re-initialize managers if system info changed significantly? (Optional)
            app.initialize_managers();
            detect_and_log_gpus(app);
            app.add_log("System analysis refreshed", LogLevel::Success);
        }
        _ => {} // Ignore other keys for now
    }
}