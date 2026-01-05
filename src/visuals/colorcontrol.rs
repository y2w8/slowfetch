// Color control module for slowfetch
// Provides hex color support and centralized color definitions
// Colors are loaded from config.toml at runtime

use crate::configloader::{ColorConfig, ThemeColor};
use crate::visuals::asciiengine::{AnsiColor, ApplyTerminalColor, TerminalColor};
use std::sync::{OnceLock, RwLock};

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

// Convert ThemeColor to TerminalColor for ASCII art colorization
fn theme_to_terminal_color(color: ThemeColor) -> TerminalColor {
    match color {
        ThemeColor::Rgb(r, g, b) => TerminalColor::Rgb(r, g, b),
        ThemeColor::Ansi(code) => TerminalColor::Ansi(ansi_code_to_color(code)),
    }
}

// Get ASCII art colors as TerminalColor array
// Returns 8 colors at indices 0-7 for {0} through {7} placeholders
pub fn get_art_colors() -> Vec<TerminalColor> {
    let c = colors();
    vec![
        theme_to_terminal_color(c.art_1),
        theme_to_terminal_color(c.art_2),
        theme_to_terminal_color(c.art_3),
        theme_to_terminal_color(c.art_4),
        theme_to_terminal_color(c.art_5),
        theme_to_terminal_color(c.art_6),
        theme_to_terminal_color(c.art_7),
        theme_to_terminal_color(c.art_8),
    ]
}

// Helper to apply a ThemeColor to text
fn apply_color(text: &str, color: ThemeColor) -> String {
    match color {
        ThemeColor::Rgb(r, g, b) => text.with_rgb(r, g, b).to_string(),
        ThemeColor::Ansi(code) => text.with_ansi(ansi_code_to_color(code)).to_string(),
    }
}

// Convert ANSI code (0-15) to AnsiColor enum
fn ansi_code_to_color(code: u8) -> AnsiColor {
    match code {
        0 => AnsiColor::Black,
        1 => AnsiColor::Red,
        2 => AnsiColor::Green,
        3 => AnsiColor::Yellow,
        4 => AnsiColor::Blue,
        5 => AnsiColor::Magenta,
        6 => AnsiColor::Cyan,
        7 => AnsiColor::White,
        8 => AnsiColor::BrightBlack,
        9 => AnsiColor::BrightRed,
        10 => AnsiColor::BrightGreen,
        11 => AnsiColor::BrightYellow,
        12 => AnsiColor::BrightBlue,
        13 => AnsiColor::BrightMagenta,
        14 => AnsiColor::BrightCyan,
        15 => AnsiColor::BrightWhite,
        _ => AnsiColor::White, // fallback
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
