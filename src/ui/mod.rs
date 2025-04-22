// Terminal UI module for Exliar VFIO Automation Framework
//
// This module provides a stylish, "girly pop" inspired Terminal UI with
// pastel colors and a clean interface for interacting with the framework

pub mod colors;
pub mod components;
pub mod terminal;
pub mod screen; // Keep old screen system for now, maybe for CLI fallback?
pub mod tui;    // New Ratatui implementation

use std::io;

/// Runs the ratatui-based UI
pub fn run_tui() -> io::Result<()> {
    tui::run_app()
}

/// Returns true if the implementation should use ratatui TUI
/// This can be controlled by environment variables or command line args
pub fn use_tui() -> bool {
    // For now, always use ratatui TUI if --ui flag is present or default
    // This could be made configurable in the future
    true
}

// Keep the old TerminalApp structure for potential CLI fallback or reference
use std::time::{Duration, Instant};
use terminal::{Terminal, TerminalEvent};
use screen::{Screen, ScreenAction, WelcomeScreen};

/// Main application for the (old) terminal UI
pub struct TerminalApp {
    screens: Vec<Box<dyn Screen>>,
    current_screen_index: usize,
    running: bool,
    terminal: Terminal,
    screen_history: Vec<usize>,
}

impl TerminalApp {
    /// Creates a new terminal application
    pub fn new() -> Self {
        Self {
            screens: Vec::new(),
            current_screen_index: 0,
            running: false,
            terminal: Terminal::new(),
            screen_history: Vec::new(),
        }
    }
    
    /// Adds a screen to the application
    pub fn add_screen(&mut self, screen: Box<dyn Screen>) {
        self.screens.push(screen);
    }
    
    /// Get the current screen
    pub fn current_screen(&self) -> Option<&Box<dyn Screen>> {
        self.screens.get(self.current_screen_index)
    }
    
    /// Get mutable reference to the current screen
    pub fn current_screen_mut(&mut self) -> Option<&mut Box<dyn Screen>> {
        self.screens.get_mut(self.current_screen_index)
    }
    
    /// Navigate to a new screen
    pub fn navigate_to(&mut self, screen: Box<dyn Screen>) {
        // Deactivate current screen
        if let Some(current) = self.current_screen_mut() {
            current.on_deactivate();
        }
        
        // Save current index for back navigation
        self.screen_history.push(self.current_screen_index);
        
        // Add and activate new screen
        self.screens.push(screen);
        self.current_screen_index = self.screens.len() - 1;
        
        if let Some(new_screen) = self.current_screen_mut() {
            new_screen.on_activate();
        }
    }
    
    /// Navigate back to the previous screen
    pub fn navigate_back(&mut self) -> bool {
        if let Some(prev_index) = self.screen_history.pop() {
            // Deactivate current screen
            if let Some(current) = self.current_screen_mut() {
                current.on_deactivate();
            }
            
            self.current_screen_index = prev_index;
            
            // Activate previous screen
            if let Some(prev_screen) = self.current_screen_mut() {
                prev_screen.on_activate();
            }
            
            true
        } else {
            false
        }
    }
    
    /// Starts the (old) terminal UI
    pub fn run(&mut self) -> io::Result<()> {
        if self.screens.is_empty() {
            eprintln!("No screens available for old UI");
            return Ok(());
        }
        
        // Enter raw mode and set up terminal
        self.terminal.enter_raw_mode()?;
        self.terminal.clear_screen()?;
        
        self.running = true;
        
        // Activate the first screen
        if let Some(screen) = self.current_screen_mut() {
            screen.on_activate();
        }
        
        let mut last_update = Instant::now();
        
        while self.running {
            // Calculate frame time
            let now = Instant::now();
            let delta = now.duration_since(last_update);
            last_update = now;
            
            // Get terminal size
            let (width, height) = self.terminal.size()?;
            
            // Update the current screen
            if let Some(screen) = self.current_screen_mut() {
                match screen.update(delta) {
                    ScreenAction::None => {},
                    ScreenAction::Navigate(new_screen) => {
                        self.navigate_to(new_screen);
                    },
                    ScreenAction::Back => {
                        if !self.navigate_back() && self.screens.len() > 1 {
                            self.running = false;
                        }
                    },
                    ScreenAction::Exit => {
                        self.running = false;
                    },
                }
            }
            
            // Render the current screen
            if let Some(screen) = self.current_screen() {
                screen.render(&self.terminal, width, height)?;
            }
            
            // Poll for events with a short timeout
            match self.terminal.poll_event(Duration::from_millis(16))? {
                TerminalEvent::KeyPress(key) => {
                    // Check for global exit key combination
                    if Terminal::is_exit_key(key) {
                        self.running = false;
                        continue;
                    }
                    
                    // Let the current screen handle the key
                    if let Some(screen) = self.current_screen_mut() {
                        match screen.handle_input(key) {
                            ScreenAction::None => {},
                            ScreenAction::Navigate(new_screen) => {
                                self.navigate_to(new_screen);
                            },
                            ScreenAction::Back => {
                                if !self.navigate_back() && self.screens.len() > 1 {
                                    self.running = false;
                                }
                            },
                            ScreenAction::Exit => {
                                self.running = false;
                            },
                        }
                    }
                },
                TerminalEvent::Resize(_, _) => {
                    // Terminal size changed, clear and re-render
                    self.terminal.clear_screen()?;
                },
                TerminalEvent::None => {},
            }
        }
        
        // Restore terminal state
        self.terminal.leave_raw_mode()?;
        self.terminal.clear_screen()?;
        
        Ok(())
    }
}

/// Creates and returns a configured (old) terminal application
pub fn create_app() -> TerminalApp {
    let mut app = TerminalApp::new();
    app.add_screen(Box::new(WelcomeScreen::new()));
    app
}