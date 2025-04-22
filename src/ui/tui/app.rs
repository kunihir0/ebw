// Main application loop for the TUI

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::core::system::SystemInfo;
// Rename the imported function to avoid conflict
use crate::gpu::detection::detect_gpus as detect_gpus_backend;

use super::state::{AppState, LogLevel};
use super::render::ui; // Explicitly import ui function
use super::input::handle_key_event;

/// Detect system information with progress messages
pub(super) fn detect_system_info(app: &mut AppState) { // Make function visible to parent module
    app.add_log("Starting system detection...", LogLevel::Info);
    app.loading_message = Some("Detecting system information...".to_string());

    // Collect system information
    let sys_info = SystemInfo::detect();

    // Log key information from the collected data
    let kernel_version = sys_info.kernel_version.full_version.clone();
    let bootloader_info = format!("{:?}", sys_info.bootloader);
    let virtualization_enabled = sys_info.virtualization_enabled;

    // Store the system info in the app state
    app.system_info = Some(sys_info); // sys_info is moved here
    app.loading_message = None;

    // Add log messages about what we found
    app.add_log("System information collected successfully", LogLevel::Success);
    app.add_log(&format!("Detected kernel: {}", kernel_version), LogLevel::Info);
    app.add_log(&format!("Bootloader: {}", bootloader_info), LogLevel::Info);

    // Add distribution info if available
    // We need to access the stored system_info now
    if let Some(sys) = &app.system_info {
        if let Some(distro) = &sys.distribution {
            let distro_info = format!("{} {}", distro.name, distro.version);
            app.add_log(&format!("Distribution: {}", distro_info), LogLevel::Info);
        }

        // Log virtualization status
        if virtualization_enabled { // Use the variable captured earlier
            app.add_log("Virtualization is enabled", LogLevel::Success);
        } else {
            app.add_log("Warning: Virtualization is disabled", LogLevel::Warning);
        }
    }
}

/// Detect GPU information with progress messages (renamed function)
pub(super) fn detect_and_log_gpus(app: &mut AppState) { // Make function visible to parent module
    app.add_log("Starting GPU detection...", LogLevel::Info);
    app.loading_message = Some("Detecting GPU devices...".to_string());
    let gpu_list = detect_gpus_backend(); // Use renamed backend function
    app.loading_message = None;

    if gpu_list.is_empty() {
        app.add_log("No GPU devices detected!", LogLevel::Warning);
    } else {
        app.add_log(&format!("Found {} GPU device(s)", gpu_list.len()), LogLevel::Success);

        // Log some key GPU details
        for (i, gpu) in gpu_list.iter().enumerate() {
            app.add_log(&format!("GPU {}: {} ({})", i+1, gpu.model_name(), gpu.vendor()), LogLevel::Info);

            // Check for potential issues
            if gpu.capabilities.has_reset_bug {
                app.add_log(&format!("Warning: GPU {} is affected by reset bug", i+1), LogLevel::Warning);
            }
            if gpu.capabilities.needs_code_43_workaround {
                app.add_log(&format!("Note: GPU {} needs Code 43 workaround", i+1), LogLevel::Info);
            }
        }
    }

    app.gpus = Some(gpu_list);
}


/// Run the ratatui app (Make this function public)
pub fn run_app() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = AppState::default();

    // Add initial welcome message
    app.add_log("Welcome to Exliar VFIO Automation", LogLevel::Info);
    app.add_log("✨ Starting system analysis...", LogLevel::Info);

    // Automatically detect system and GPUs on startup
    terminal.draw(|f| ui(f, &app))?; // Draw initial loading state
    detect_system_info(&mut app); // Call the function
    app.initialize_managers(); // Initialize managers after detecting system info
    terminal.draw(|f| ui(f, &app))?; // Draw after system info
    detect_and_log_gpus(&mut app); // Call the renamed function
    terminal.draw(|f| ui(f, &app))?; // Draw after GPU info
    app.add_log("System analysis complete. Press 'g' for GPU details, 'r' to refresh, 'q' to quit.", LogLevel::Success);

    // Run main interactive loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle timeout and input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Pass app state to input handler
                handle_key_event(&mut app, key.code, key.modifiers);
            }
        }

        // Check if we need to quit
        if app.should_quit {
            break;
        }

        // Update tick rate
        if last_tick.elapsed() >= tick_rate {
            // Placeholder for potential future tick-based updates
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}