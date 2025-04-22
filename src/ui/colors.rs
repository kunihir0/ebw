// Pastel color palette for the "girly pop" aesthetic UI

use crossterm::style::{Color, Stylize, StyledContent};
use std::fmt;

/// Defines the pastel color palette for the UI
#[derive(Clone, Copy, Debug)]
pub enum PastelColor {
    Pink,
    Lavender,
    Mint,
    SkyBlue,
    Peach,
    LightYellow,
    White,
    Gray,
}

impl PastelColor {
    /// Get the terminal color representation
    pub fn as_color(&self) -> Color {
        match self {
            PastelColor::Pink => Color::Rgb { r: 255, g: 182, b: 193 },       // Light pink
            PastelColor::Lavender => Color::Rgb { r: 204, g: 169, b: 221 },   // Light purple
            PastelColor::Mint => Color::Rgb { r: 176, g: 224, b: 183 },       // Mint green
            PastelColor::SkyBlue => Color::Rgb { r: 173, g: 216, b: 230 },    // Light sky blue
            PastelColor::Peach => Color::Rgb { r: 255, g: 218, b: 185 },      // Peach
            PastelColor::LightYellow => Color::Rgb { r: 255, g: 255, b: 224 },// Light yellow
            PastelColor::White => Color::White,
            PastelColor::Gray => Color::Rgb { r: 169, g: 169, b: 169 },       // Light gray
        }
    }
}

/// A styled text element with type safety
pub struct StyledText<'a> {
    content: &'a str,
    styled: StyledContent<&'a str>,
}

impl<'a> StyledText<'a> {
    /// Create new styled text with foreground color
    pub fn new(text: &'a str, fg_color: PastelColor) -> Self {
        Self {
            content: text,
            styled: text.with(fg_color.as_color()),
        }
    }
    
    /// Create new styled text with foreground and background color
    pub fn with_bg(text: &'a str, fg_color: PastelColor, bg_color: PastelColor) -> Self {
        Self {
            content: text,
            styled: text.with(fg_color.as_color()).on(bg_color.as_color()),
        }
    }
    
    /// Create bold styled text
    pub fn bold(text: &'a str, fg_color: PastelColor) -> Self {
        Self {
            content: text,
            styled: text.with(fg_color.as_color()).bold(),
        }
    }
    
    /// Get the raw text content
    pub fn content(&self) -> &str {
        self.content
    }
}

impl<'a> fmt::Display for StyledText<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.styled)
    }
}

/// Theme defining the main colors used by the UI
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub primary: PastelColor,
    pub secondary: PastelColor,
    pub accent: PastelColor,
    pub text: PastelColor,
    pub background: PastelColor,
    pub error: PastelColor,
    pub success: PastelColor,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: PastelColor::Lavender,
            secondary: PastelColor::SkyBlue,
            accent: PastelColor::Pink,
            text: PastelColor::White,
            background: PastelColor::White,
            error: PastelColor::Peach,
            success: PastelColor::Mint,
        }
    }
}