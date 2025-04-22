// Application state management for the TUI

use crate::core::system::SystemInfo;
use crate::gpu::GpuDevice;
use crate::core::vfio::VfioManager;
use crate::core::state::StateTracker;
use crate::core::bootloader::{BootloaderManager, get_bootloader_manager};
use ratatui::style::Color;
use std::path::PathBuf;

/// A styled log message for the console feed
#[derive(Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub text: String,
    pub level: LogLevel,
}

/// Log message levels with associated colors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    #[allow(dead_code)] // Allow unused variant for now
    Error,
}

impl LogLevel {
    /// Get the color for this log level
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Info => Color::Rgb(204, 169, 221),    // Lavender
            LogLevel::Success => Color::Rgb(176, 224, 183), // Mint
            LogLevel::Warning => Color::Rgb(255, 218, 185), // Peach
            LogLevel::Error => Color::Rgb(255, 182, 193),   // Pink
        }
    }
}

/// Ratatui app state
#[allow(dead_code)] // Allow unused fields for now
pub struct AppState {
    pub title: String,
    pub should_quit: bool,
    pub system_info: Option<SystemInfo>,
    pub gpus: Option<Vec<GpuDevice>>,
    pub loading_message: Option<String>,
    pub log_messages: Vec<LogMessage>,
    pub selected_gpu_index: usize, // Index for GPU detail view / selection
    pub show_gpu_details: bool,
    // Core managers (initialized later) - Make fields pub
    pub vfio_manager: Option<VfioManager>,
    pub bootloader_manager: Option<Box<dyn BootloaderManager>>,
    pub state_tracker: Option<StateTracker>,
    // State related to configuration - Make fields pub
    pub selected_passthrough_gpu_index: Option<usize>, // Index of GPU selected for passthrough
    pub configuration_applied: bool, // Track if initial config steps done
    pub reboot_required: bool,
    pub current_action: Option<String>, // To show what action is being performed
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            title: "Exliar VFIO".to_string(),
            should_quit: false,
            system_info: None,
            gpus: None,
            loading_message: None,
            log_messages: Vec::new(),
            selected_gpu_index: 0,
            show_gpu_details: false,
            vfio_manager: None,
            bootloader_manager: None,
            state_tracker: None,
            selected_passthrough_gpu_index: None, // Initialize as None
            configuration_applied: false,
            reboot_required: false,
            current_action: None,
        }
    }
}

impl AppState {
    /// Add a log message to the console feed
    pub fn add_log(&mut self, text: &str, level: LogLevel) {
        // Get current time
        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S").to_string();

        // Add message to log
        self.log_messages.push(LogMessage {
            timestamp,
            text: text.to_string(),
            level,
        });

        // Keep log at reasonable size
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }

    /// Initializes the core managers after system info is detected
    pub fn initialize_managers(&mut self) {
        if let Some(sys_info) = &self.system_info {
            // Clone necessary parts of sys_info or the whole struct if needed by managers
            let cloned_sys_info = sys_info.clone();
            self.vfio_manager = Some(VfioManager::new(cloned_sys_info)); // Pass cloned info
            self.bootloader_manager = get_bootloader_manager(&sys_info.bootloader);

            // Initialize StateTracker (define path for state file)
            // TODO: Make state file path configurable
            let state_path = PathBuf::from(".exliar_state.json");
            match StateTracker::new(state_path) {
                Ok(tracker) => self.state_tracker = Some(tracker),
                Err(e) => self.add_log(&format!("Failed to initialize state tracker: {}", e), LogLevel::Error),
            }

            self.add_log("Core managers initialized.", LogLevel::Info);
        } else {
            self.add_log("Cannot initialize managers: System info not detected.", LogLevel::Error);
        }
    }

    /// Get the currently selected GPU for passthrough, if any
    #[allow(dead_code)] // Allow unused function for now
    pub fn get_selected_passthrough_gpu(&self) -> Option<&GpuDevice> {
        if let (Some(idx), Some(gpus)) = (self.selected_passthrough_gpu_index, &self.gpus) {
            gpus.get(idx)
        } else {
            None
        }
    }
}