// Terminal handling for TUI application

use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
    ExecutableCommand,
};

/// Result type for terminal operations
pub type Result<T> = std::result::Result<T, io::Error>;

/// Terminal event types
pub enum TerminalEvent {
    /// Key press event with the pressed key
    KeyPress(KeyEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// No event (timeout)
    None,
}

/// Terminal handler for raw mode and event processing
pub struct Terminal {
    raw_mode: bool,
}

impl Terminal {
    /// Create a new terminal handler
    pub fn new() -> Self {
        Self { raw_mode: false }
    }

    /// Enter raw mode and alternate screen
    pub fn enter_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode {
            terminal::enable_raw_mode()?;
            io::stdout()
                .execute(EnterAlternateScreen)?
                .execute(Hide)?;
            self.raw_mode = true;
        }
        Ok(())
    }

    /// Leave raw mode and alternate screen
    pub fn leave_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode {
            io::stdout()
                .execute(Show)?
                .execute(LeaveAlternateScreen)?;
            terminal::disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    /// Clear the screen
    pub fn clear_screen(&self) -> Result<()> {
        io::stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        io::stdout().execute(crossterm::cursor::MoveTo(0, 0))?;
        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> Result<(u16, u16)> {
        terminal::size()
    }

    /// Handle terminal events with timeout
    pub fn poll_event(&self, timeout: Duration) -> Result<TerminalEvent> {
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                Ok(TerminalEvent::KeyPress(key))
            } else if let Event::Resize(width, height) = event::read()? {
                Ok(TerminalEvent::Resize(width, height))
            } else {
                Ok(TerminalEvent::None)
            }
        } else {
            Ok(TerminalEvent::None)
        }
    }

    /// Print at a specific position
    pub fn print_at(&self, x: u16, y: u16, text: &str) -> Result<()> {
        io::stdout()
            .execute(crossterm::cursor::MoveTo(x, y))?;
        print!("{}", text);
        io::stdout().flush()?;
        Ok(())
    }

    /// Print a multi-line string at a specific position
    pub fn print_multi_line_at(&self, x: u16, y: u16, text: &str) -> Result<()> {
        for (i, line) in text.lines().enumerate() {
            self.print_at(x, y + i as u16, line)?;
        }
        Ok(())
    }

    /// Check if Ctrl+C was pressed
    pub fn is_exit_key(key: KeyEvent) -> bool {
        (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) ||
        (key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL)) ||
        key.code == KeyCode::Esc
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Ensure we leave raw mode when dropped
        let _ = self.leave_raw_mode();
    }
}