// Screen management for the UI

use std::time::Duration;
use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::terminal::Terminal;
use crate::ui::colors::Theme;

/// Actions that can be taken after handling input
pub enum ScreenAction {
    /// Stay on the current screen
    None,
    /// Navigate to another screen
    Navigate(Box<dyn Screen>),
    /// Go back to the previous screen
    Back,
    /// Exit the application
    Exit,
}

/// Represents a screen in the terminal UI
pub trait Screen {
    /// Returns the title of the screen
    fn title(&self) -> &str;
    
    /// Renders the screen
    fn render(&self, term: &Terminal, width: u16, height: u16) -> std::io::Result<()>;
    
    /// Handles user input
    fn handle_input(&mut self, key: KeyEvent) -> ScreenAction;
    
    /// Update screen state (animations, refreshing data, etc)
    fn update(&mut self, _delta: Duration) -> ScreenAction {
        // Default implementation does nothing
        ScreenAction::None
    }
    
    /// Called when the screen becomes active
    fn on_activate(&mut self) {}
    
    /// Called when the screen becomes inactive (another screen is shown)
    fn on_deactivate(&mut self) {}
    
    /// Get the theme for this screen (returns owned Theme since it's Copy)
    fn theme(&self) -> Theme {
        // Default implementation returns the default theme
        Theme::default()
    }
}

/// Welcome screen implementation with "girly pop" style
pub struct WelcomeScreen {
    title: String,
    theme: Theme,
    selected_option: usize,
    options: Vec<String>,
}

impl WelcomeScreen {
    /// Create a new welcome screen
    pub fn new() -> Self {
        Self {
            title: "Exliar VFIO".to_string(),
            theme: Theme::default(),
            selected_option: 0,
            options: vec![
                "Detect System".to_string(),
                "Detect GPUs".to_string(),
                "Configure VFIO".to_string(),
                "Exit".to_string(),
            ],
        }
    }
}

impl Screen for WelcomeScreen {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn render(&self, term: &Terminal, width: u16, height: u16) -> std::io::Result<()> {
        // Clear screen
        term.clear_screen()?;
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 3;
        
        // Draw a pretty title
        let title = "✨ Exliar VFIO ✨";
        term.print_at(center_x - (title.len() as u16 / 2), center_y - 4, title)?;
        
        let subtitle = "GPU Passthrough Automation";
        term.print_at(center_x - (subtitle.len() as u16 / 2), center_y - 2, subtitle)?;
        
        // Draw options
        for (i, option) in self.options.iter().enumerate() {
            let prefix = if i == self.selected_option { "➤ " } else { "  " };
            let option_text = format!("{}{}", prefix, option);
            
            // Calculate position
            let option_x = center_x - (option_text.len() as u16 / 2);
            let option_y = center_y + (i as u16 * 2) + 2;
            
            // Add some color for the selected option
            if i == self.selected_option {
                // Use crossterm's styling
                use crossterm::style::Stylize;
                let styled = option_text.magenta().bold();
                term.print_at(option_x, option_y, &format!("{}", styled))?;
            } else {
                term.print_at(option_x, option_y, &option_text)?;
            }
        }
        
        // Draw footer
        let footer = "↑/↓: Navigate | Enter: Select | Esc: Exit";
        term.print_at(center_x - (footer.len() as u16 / 2), height - 2, footer)?;
        
        Ok(())
    }
    
    fn handle_input(&mut self, key: KeyEvent) -> ScreenAction {
        match key.code {
            KeyCode::Up => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                } else {
                    self.selected_option = self.options.len() - 1;
                }
                ScreenAction::None
            }
            KeyCode::Down => {
                self.selected_option = (self.selected_option + 1) % self.options.len();
                ScreenAction::None
            }
            KeyCode::Enter => {
                match self.selected_option {
                    0 => ScreenAction::None, // Would navigate to system detection
                    1 => ScreenAction::None, // Would navigate to GPU detection
                    2 => ScreenAction::None, // Would navigate to VFIO configuration
                    3 => ScreenAction::Exit,
                    _ => ScreenAction::None,
                }
            }
            KeyCode::Esc => ScreenAction::Exit,
            _ => ScreenAction::None,
        }
    }
    
    fn theme(&self) -> Theme {
        self.theme
    }
}