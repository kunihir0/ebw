# UI Implementation Plan for Exliar VFIO

## Current Issues to Fix
1. Unused assignments in `src/gpu/detection.rs`
2. Unused import in `src/plugin/mod.rs`
3. Unused field in `src/ui/mod.rs`

## New UI Library Design

### Aesthetic & Design Goals
- Pastel color palette (lavender, mint, soft pink, baby blue)
- Clean, minimal interface with thoughtful spacing
- Type-safe design using Rust's type system
- "Girly pop" aesthetic with subtle, tasteful design elements
- Professional yet approachable look and feel

### Technical Implementation

#### Core Components:
1. **Color System**
   - Define a pastel color palette as constants
   - Create a type-safe color application system
   - Support for foreground/background color combinations

2. **Text Styling**
   - Bold, italic, underline options
   - Text alignment (left, center, right)
   - Borders and box drawing with customizable styles

3. **Layout Components**
   - Panels with customizable borders and colors
   - Lists with selection highlighting
   - Progress indicators
   - Headers and status bars

4. **Input Handling**
   - Type-safe keyboard event system
   - Selection and navigation interfaces
   - Form inputs with validation

5. **Screen Management**
   - Improved navigation between screens with history
   - Transitions between screens
   - Proper utilization of current_screen_index

### Implementation Strategy
1. Create basic terminal control layer (using crossterm)
2. Implement the color and styling system
3. Build layout components
4. Implement input handling
5. Create screen management system
6. Replace existing UI implementation while maintaining the same public API

### Dependencies
- `crossterm` for terminal control
- `unicode-width` for proper text alignment

## Directory Structure
```
src/ui/
├── mod.rs           # Public API
├── color.rs         # Color system
├── style.rs         # Text styling
├── components/      # UI components
│   ├── mod.rs
│   ├── panel.rs
│   ├── list.rs
│   └── ...
├── input.rs         # Input handling
├── screen.rs        # Screen trait and management
└── terminal.rs      # Terminal control
```

## Timeline
1. Fix compiler warnings (immediate)
2. Implement color and styling system (short-term)
3. Build components and layout system (medium-term)
4. Integrate input handling and screen management (medium-term)
5. Create demo screens to showcase the library (long-term)