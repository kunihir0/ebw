// List component for creating selectable lists in the UI

use crate::ui::colors::{StyledText, Theme};
use unicode_width::UnicodeWidthStr;

/// Represents a selectable list in the UI
pub struct List<T> {
    items: Vec<T>,
    selected_index: usize,
    theme: Theme,
    width: usize,
}

impl<T> List<T> 
where
    T: AsRef<str> + Clone,
{
    /// Create a new list with default theme
    pub fn new(width: usize) -> Self {
        Self::with_theme(width, Theme::default())
    }

    /// Create a new list with specified theme
    pub fn with_theme(width: usize, theme: Theme) -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            theme,
            width,
        }
    }

    /// Add an item to the list
    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    /// Set items from an iterator
    pub fn set_items<I>(&mut self, items: I) 
    where
        I: IntoIterator<Item = T>,
    {
        self.items = items.into_iter().collect();
        self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&T> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.selected_index])
        }
    }

    /// Move the selection up
    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Move the selection down
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }

    /// Get the current selection index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set the selection index
    pub fn set_selected_index(&mut self, index: usize) {
        if !self.items.is_empty() {
            self.selected_index = index % self.items.len();
        }
    }

    /// Get all items
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Render the list as a string
    pub fn render(&self) -> String {
        let mut result = String::new();

        // Calculate max visible items based on height
        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected_index;
            
            // Create the prefix based on selection state
            let prefix = if is_selected {
                StyledText::with_bg(" ✦ ", self.theme.accent, self.theme.primary).to_string()
            } else {
                "   ".to_string()
            };
            
            // Get the item text
            let item_text = item.as_ref();
            let item_width = UnicodeWidthStr::width(item_text);
            
            // Format the line with prefix and item
            let mut line = String::new();
            line.push_str(&prefix);
            
            // Display the item, possibly truncating
            if item_width > self.width - 4 {
                // Need to truncate
                let mut displayable = String::new();
                let mut current_width = 0;
                
                for c in item_text.chars() {
                    let char_width = UnicodeWidthStr::width(c.to_string().as_str());
                    if current_width + char_width <= self.width - 7 {
                        displayable.push(c);
                        current_width += char_width;
                    } else {
                        break;
                    }
                }
                
                line.push_str(&displayable);
                line.push_str("...");
            } else {
                line.push_str(item_text);
                
                // Pad to full width if needed
                for _ in 0..(self.width - 4 - item_width) {
                    line.push(' ');
                }
            }
            
            // Style the line based on selection state
            if is_selected {
                result.push_str(&StyledText::bold(&line, self.theme.accent).to_string());
            } else {
                result.push_str(&line);
            }
            
            if i < self.items.len() - 1 {
                result.push('\n');
            }
        }
        
        result
    }
}