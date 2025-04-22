// Border component for drawing boxes and borders around UI elements

use crate::ui::colors::{PastelColor, StyledText};
use unicode_width::UnicodeWidthStr;

/// Border style for UI components
pub enum BorderStyle {
    /// Single line border (────┐)
    Single,
    /// Double line border (════╗)
    Double,
    /// Rounded border (────╮)
    Rounded,
    /// No visible border
    None,
}

/// Represents a border that can be drawn around UI elements
pub struct Border {
    style: BorderStyle,
    color: PastelColor,
    title: Option<String>,
}

impl Border {
    /// Create a new border with the specified style and color
    pub fn new(style: BorderStyle, color: PastelColor) -> Self {
        Self {
            style,
            color,
            title: None,
        }
    }
    
    /// Add a title to the border
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    
    /// Draw the top part of the border with optional title
    pub fn draw_top(&self, width: usize) -> String {
        let (left, horizontal, right) = match self.style {
            BorderStyle::Single => ("┌", "─", "┐"),
            BorderStyle::Double => ("╔", "═", "╗"),
            BorderStyle::Rounded => ("╭", "─", "╮"),
            BorderStyle::None => (" ", " ", " "),
        };
        
        let mut result = String::new();
        
        // Add the left corner
        result.push_str(&StyledText::new(left, self.color).to_string());
        
        // If there's a title, draw it in the middle
        if let Some(title) = &self.title {
            let title_len = UnicodeWidthStr::width(title.as_str());
            
            // Calculate padding for centering the title
            let available_width = if width > 2 { width - 2 } else { 0 };
            let padding = if title_len < available_width {
                (available_width - title_len) / 2
            } else {
                0
            };
            
            // Add left padding
            for _ in 0..padding {
                result.push_str(&StyledText::new(horizontal, self.color).to_string());
            }
            
            // Add title
            result.push_str(&StyledText::bold(title, self.color).to_string());
            
            // Add right padding
            let remaining = available_width - padding - title_len;
            for _ in 0..remaining {
                result.push_str(&StyledText::new(horizontal, self.color).to_string());
            }
        } else {
            // No title, just draw a horizontal line
            for _ in 0..(width - 2) {
                result.push_str(&StyledText::new(horizontal, self.color).to_string());
            }
        }
        
        // Add the right corner
        result.push_str(&StyledText::new(right, self.color).to_string());
        
        result
    }
    
    /// Draw a horizontal line for the bottom of the border
    pub fn draw_bottom(&self, width: usize) -> String {
        let (left, horizontal, right) = match self.style {
            BorderStyle::Single => ("└", "─", "┘"),
            BorderStyle::Double => ("╚", "═", "╝"),
            BorderStyle::Rounded => ("╰", "─", "╯"),
            BorderStyle::None => (" ", " ", " "),
        };
        
        let mut result = String::new();
        
        // Add the left corner
        result.push_str(&StyledText::new(left, self.color).to_string());
        
        // Draw a horizontal line
        for _ in 0..(width - 2) {
            result.push_str(&StyledText::new(horizontal, self.color).to_string());
        }
        
        // Add the right corner
        result.push_str(&StyledText::new(right, self.color).to_string());
        
        result
    }
    
    /// Draw vertical borders for content
    pub fn draw_content_line(&self, content: &str, width: usize) -> String {
        let vertical = match self.style {
            BorderStyle::Single => "│",
            BorderStyle::Double => "║",
            BorderStyle::Rounded => "│",
            BorderStyle::None => " ",
        };
        
        let mut result = String::new();
        
        // Add left vertical line
        result.push_str(&StyledText::new(vertical, self.color).to_string());
        
        // Add content with padding if needed
        let content_width = UnicodeWidthStr::width(content);
        if content_width <= width - 2 {
            result.push_str(content);
            
            // Add padding spaces
            for _ in 0..(width - 2 - content_width) {
                result.push(' ');
            }
        } else {
            // Content is too long, truncate it
            let mut displayable = String::new();
            let mut current_width = 0;
            
            for c in content.chars() {
                let char_width = UnicodeWidthStr::width(c.to_string().as_str());
                if current_width + char_width <= width - 5 {
                    displayable.push(c);
                    current_width += char_width;
                } else {
                    break;
                }
            }
            
            result.push_str(&displayable);
            result.push_str("...");
            
            // Add padding spaces if any left
            for _ in 0..(width - 2 - current_width - 3) {
                result.push(' ');
            }
        }
        
        // Add right vertical line
        result.push_str(&StyledText::new(vertical, self.color).to_string());
        
        result
    }
}