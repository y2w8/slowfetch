// Color control module for slowfetch
// Provides hex color support and centralized color definitions
// Colors are loaded from config.toml at runtime

use crate::configloader::{ColorConfig, ThemeColor};
use std::sync::{OnceLock, RwLock};
use tintify::{DynColors, TintColorize, AnsiColors};

// Global color config, initialized once from config file
static COLORS: OnceLock<ColorConfig> = OnceLock::new();

// Override colors for preview mode (can be changed at runtime)
static PREVIEW_COLORS: RwLock<Option<ColorConfig>> = RwLock::new(None);

// Initialize colors from config - call this once at startup
pub fn init_colors(colors: ColorConfig) {
    let _ = COLORS.set(colors);
}

// Set preview colors (for TUI config preview)
pub fn set_preview_colors(colors: Option<ColorConfig>) {
    if let Ok(mut preview) = PREVIEW_COLORS.write() {
        *preview = colors;
    }
}

// Get the current color config (preview colors override base colors)
fn colors() -> ColorConfig {
    if let Ok(preview) = PREVIEW_COLORS.read() {
        if let Some(ref c) = *preview {
            return c.clone();
        }
    }
    COLORS.get_or_init(ColorConfig::default).clone()
}

// Convert ThemeColor to DynColors for inkline
fn theme_to_dyn(color: ThemeColor) -> DynColors {
    match color {
        ThemeColor::Rgb(r, g, b) => DynColors::Rgb(r, g, b),
        ThemeColor::Ansi(code) => DynColors::Ansi(ansi_to_color(code)),
    }
}

// Get ASCII art colors as DynColors array for inkline
// Returns 8 colors at indices 0-7 for inkline's {0} through {7} placeholders
pub fn get_art_colors() -> Vec<DynColors> {
    let c = colors();
    vec![
        theme_to_dyn(c.art_1),
        theme_to_dyn(c.art_2),
        theme_to_dyn(c.art_3),
        theme_to_dyn(c.art_4),
        theme_to_dyn(c.art_5),
        theme_to_dyn(c.art_6),
        theme_to_dyn(c.art_7),
        theme_to_dyn(c.art_8),
    ]
}

// Helper to apply a ThemeColor to text
fn apply_color(text: &str, color: ThemeColor) -> String {
    match color {
        ThemeColor::Rgb(r, g, b) => text.truecolor(r, g, b).to_string(),
        ThemeColor::Ansi(code) => text.color(ansi_to_color(code)).to_string(),
    }
}

// Convert ANSI code to AnsiColors
fn ansi_to_color(code: u8) -> AnsiColors {
    match code {
        0 => AnsiColors::Black,
        1 => AnsiColors::Red,
        2 => AnsiColors::Green,
        3 => AnsiColors::Yellow,
        4 => AnsiColors::Blue,
        5 => AnsiColors::Magenta,
        6 => AnsiColors::Cyan,
        7 => AnsiColors::White,
        8 => AnsiColors::BrightBlack,
        9 => AnsiColors::BrightRed,
        10 => AnsiColors::BrightGreen,
        11 => AnsiColors::BrightYellow,
        12 => AnsiColors::BrightBlue,
        13 => AnsiColors::BrightMagenta,
        14 => AnsiColors::BrightCyan,
        15 => AnsiColors::BrightWhite,
        _ => AnsiColors::White, // fallback
    }
}

// Color application functions
pub fn color_border(text: &str) -> String {
    apply_color(text, colors().border)
}

pub fn color_title(text: &str) -> String {
    apply_color(text, colors().title)
}

pub fn color_key(text: &str) -> String {
    apply_color(text, colors().key)
}

pub fn color_value(text: &str) -> String {
    apply_color(text, colors().value)
}
