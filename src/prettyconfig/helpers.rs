// Helper functions for prettyconfig TUI
// Color conversion and widget builders

use crate::configloader::{ThemeColor, ThemePreset};
use ratatui::style::Color;

// Convert ThemeColor to ratatui Color
pub fn theme_color_to_ratatui(color: ThemeColor) -> Color {
    match color {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        ThemeColor::Ansi(code) => Color::Indexed(code),
    }
}

// Get all available theme presets for cycling
pub const THEME_PRESETS: [ThemePreset; 8] = [
    ThemePreset::Default,
    ThemePreset::Dracula,
    ThemePreset::Catppuccin,
    ThemePreset::Nord,
    ThemePreset::Gruvbox,
    ThemePreset::Eldritch,
    ThemePreset::Kanagawa,
    ThemePreset::RosePine,
];

// Get theme name for display
pub fn theme_name(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Default => "Default",
        ThemePreset::Dracula => "Dracula",
        ThemePreset::Catppuccin => "Catppuccin",
        ThemePreset::Nord => "Nord",
        ThemePreset::Gruvbox => "Gruvbox",
        ThemePreset::Eldritch => "Eldritch",
        ThemePreset::Kanagawa => "Kanagawa",
        ThemePreset::RosePine => "Rosé Pine",
    }
}

// Get next theme in cycle
pub fn next_theme(current: ThemePreset) -> ThemePreset {
    let idx = THEME_PRESETS.iter().position(|&t| std::mem::discriminant(&t) == std::mem::discriminant(&current)).unwrap_or(0);
    THEME_PRESETS[(idx + 1) % THEME_PRESETS.len()]
}

// Get previous theme in cycle
pub fn prev_theme(current: ThemePreset) -> ThemePreset {
    let idx = THEME_PRESETS.iter().position(|&t| std::mem::discriminant(&t) == std::mem::discriminant(&current)).unwrap_or(0);
    THEME_PRESETS[(idx + THEME_PRESETS.len() - 1) % THEME_PRESETS.len()]
}

