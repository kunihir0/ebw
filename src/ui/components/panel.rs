// Panel component for creating bordered sections in the UI

use crate::ui::colors::{PastelColor, StyledText, Theme};
use super::border::{Border, BorderStyle};
use unicode_width::UnicodeWidthStr;

/// Represents a panel in the UI with a border and content
pub struct Panel {
    border: Border,
    content: Vec<String>,
    width: usize,
    height: usize,
    theme: Theme,
}

impl Panel {
    /// Create a new panel with default theme
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_theme(width, height, Theme::default())
    }

    /// Create a new panel with specified theme
    pub fn with_theme(width: usize, height: usize, theme: Theme) -> Self {
        Self {
            border: Border::new(BorderStyle::Rounded, theme.primary),
            content: Vec::new(),
            width,
            height,
            theme,
        }
    }

    /// Set the border style for the panel
    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border = Border::new(style, self.theme.primary);
        self
    }

    /// Set the border color for the panel
    pub fn with_border_color(mut self, color: PastelColor) -> Self {
        self.border = Border::new(BorderStyle::Rounded, color);
        self
    }

    /// Set a title for the panel
    pub fn with_title(mut self, title: &str) -> Self {
        self.border = self.border.with_title(title);
        self
    }

    /// Add content to the panel
    pub fn add_line(&mut self, line: &str) {
        self.content.push(line.to_string());
        // Ensure we don't exceed max height (excluding borders)
        let max_content_lines = if self.height > 2 { self.height - 2 } else { 0 };
        if self.content.len() > max_content_lines {
            self.content.remove(0);
        }
    }

    /// Add styled content to the panel
    pub fn add_styled_line<'a>(&mut self, text: StyledText<'a>) {
        self.add_line(&text.to_string());
    }

    /// Add a centered line to the panel
    pub fn add_centered_line(&mut self, line: &str) {
        let content_width = if self.width > 2 { self.width - 2 } else { 0 };
        let line_width = UnicodeWidthStr::width(line);
        
        if line_width >= content_width {
            // Line is too wide, just add it as-is
            self.add_line(line);
            return;
        }
        
        // Calculate padding for centering
        let padding = (content_width - line_width) / 2;
        let mut padded_line = String::new();
        
        // Add left padding
        for _ in 0..padding {
            padded_line.push(' ');
        }
        
        // Add the content
        padded_line.push_str(line);
        
        // Add right padding if needed (for odd width differences)
        for _ in 0..(content_width - line_width - padding) {
            padded_line.push(' ');
        }
        
        self.add_line(&padded_line);
    }

    /// Add a horizontal separator line
    pub fn add_separator(&mut self) {
        let content_width = if self.width > 2 { self.width - 2 } else { 0 };
        let mut separator = String::new();
        for _ in 0..content_width {
            separator.push('─');
        }
        self.add_line(&separator);
    }

    /// Clear all content from the panel
    pub fn clear(&mut self) {
        self.content.clear();
    }

    /// Render the panel as a string
    pub fn render(&self) -> String {
        let mut result = String::new();
        
        // Draw the top border
        result.push_str(&self.border.draw_top(self.width));
        result.push('\n');
        
        // Calculate available content lines
        let content_height = if self.height > 2 { self.height - 2 } else { 0 };
        
        // Draw content lines or padding if needed
        for i in 0..content_height {
            if i < self.content.len() {
                result.push_str(&self.border.draw_content_line(&self.content[i], self.width));
            } else {
                // Draw empty content line
                result.push_str(&self.border.draw_content_line("", self.width));
            }
            result.push('\n');
        }
        
        // Draw the bottom border
        result.push_str(&self.border.draw_bottom(self.width));
        
        result
    }

    /// Get the width of the panel
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the panel
    pub fn height(&self) -> usize {
        self.height
    }
}