use exliar_vfio::core::system::SystemInfo;
use exliar_vfio::gpu::detection::detect_gpus;
use exliar_vfio::gpu::vendor::GpuVendorHandler;
use exliar_vfio::gpu::vendor::amd::AmdGpuHandler;
use exliar_vfio::gpu::vendor::nvidia::NvidiaGpuHandler;
use exliar_vfio::gpu::vendor::intel::IntelGpuHandler;
use exliar_vfio::ui; // Import the ui module
use std::env;

fn main() -> std::io::Result<()> {
    // Check if CLI mode is explicitly requested
    let args: Vec<String> = env::args().collect();
    let use_cli_flag = args.iter().any(|arg| arg == "--cli");

    if use_cli_flag {
        // Use the command-line interface
        run_cli_mode();
        Ok(())
    } else {
        // Default to the ratatui-based UI
        println!("Starting Exliar VFIO TUI..."); // Optional: Keep or remove this line
        ui::run_tui() // Run the TUI by default
    }
}

/// Run the traditional command-line interface mode
fn run_cli_mode() {
    println!("Exliar VFIO Automation Framework (CLI Mode)");
    println!("===========================================\n");
    println!("Version: {}", exliar_vfio::VERSION);

    // Detect system information
    println!("\nDetecting system information...");
    let system_info = SystemInfo::detect();
    println!("{}", system_info.summary());

    // Detect GPUs
    println!("\nDetecting GPUs...");
    let gpus = detect_gpus();

    if gpus.is_empty() {
        println!("No GPUs detected!");
        return;
    }

    println!("Found {} GPU(s):", gpus.len());

    // Create vendor handlers
    let handlers: Vec<Box<dyn GpuVendorHandler>> = vec![
        Box::new(AmdGpuHandler),
        Box::new(NvidiaGpuHandler),
        Box::new(IntelGpuHandler),
    ];

    // Process each GPU
    for (i, gpu) in gpus.iter().enumerate() {
        println!("\nGPU {}: {} ({})", i+1, gpu.model_name(), gpu.vendor());
        println!("  BDF: {}", gpu.bdf());
        println!("  Vendor ID: {}", gpu.vendor_id);
        println!("  Device ID: {}", gpu.device_id);
        println!("  Driver: {}", gpu.driver.as_deref().unwrap_or("None"));
        println!("  Integrated: {}", if gpu.is_integrated { "Yes" } else { "No" });

        // Print capabilities
        println!("  Capabilities:");
        println!("    Reset Support: {}", if gpu.capabilities.supports_reset { "Yes" } else { "No" });
        println!("    Reset Bug: {}", if gpu.capabilities.has_reset_bug { "Yes (affected)" } else { "No" });
        println!("    Code 43 Workaround Needed: {}",
                if gpu.capabilities.needs_code_43_workaround { "Yes" } else { "No" });
        println!("    GVT-g Support: {}", if gpu.capabilities.supports_gvt { "Yes" } else { "No" });

        // Find appropriate handler
        let handler = handlers.iter().find(|h| h.supports_device(gpu));

        if let Some(handler) = handler {
            println!("\n  Using handler: {}", handler.name());

            // Prepare for passthrough (prints warnings/info)
            println!("\n  Preparation information:");
            match handler.prepare_for_passthrough(gpu) {
                Ok(_) => {},
                Err(e) => println!("    Error during preparation: {}", e),
            }

            // Get quirks
            println!("\n  Required quirks:");
            match handler.apply_quirks(gpu) {
                Ok(quirks) => {
                    if quirks.is_empty() {
                        println!("    None required");
                    } else {
                        for quirk in quirks {
                            println!("    - {}: {}", quirk.name, quirk.description);
                        }
                    }
                },
                Err(e) => println!("    Error determining quirks: {}", e),
            }

            // Check passthrough readiness
            println!("\n  Passthrough readiness check:");
            match handler.verify_passthrough_ready(gpu) {
                Ok(true) => println!("    GPU is ready for passthrough"),
                Ok(false) => println!("    GPU is NOT ready for passthrough"),
                Err(e) => println!("    Error checking passthrough readiness: {}", e),
            }
        } else {
            println!("\n  No handler found for this GPU");
        }

        println!("\n{}", "-".repeat(50));
    }

    println!("\nExliar VFIO Automation Framework setup complete");
}
