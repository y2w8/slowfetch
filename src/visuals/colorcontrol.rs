// Color control module for slowfetch
// Provides hex color support and centralized color definitions
// Colors are loaded from config.toml at runtime

use crate::configloader::{ColorConfig, ThemeColor};
use std::sync::OnceLock;
use tintify::{DynColors, TintColorize, AnsiColors};

// Global color config, initialized once from config file
static COLORS: OnceLock<ColorConfig> = OnceLock::new();

// Initialize colors from config - call this once at startup
pub fn init_colors(colors: ColorConfig) {
    let _ = COLORS.set(colors);
}

// Get the current color config
fn colors() -> &'static ColorConfig {
    COLORS.get_or_init(ColorConfig::default)
}

// Get ASCII art colors as DynColors array for inkline
pub fn get_art_colors() -> Vec<DynColors> {
    let c = colors();
    vec![
        DynColors::Rgb(c.art_1.0, c.art_1.1, c.art_1.2),
        DynColors::Rgb(c.art_2.0, c.art_2.1, c.art_2.2),
        DynColors::Rgb(c.art_3.0, c.art_3.1, c.art_3.2),
        DynColors::Rgb(c.art_4.0, c.art_4.1, c.art_4.2),
        DynColors::Rgb(c.art_5.0, c.art_5.1, c.art_5.2),
        DynColors::Rgb(c.art_6.0, c.art_6.1, c.art_6.2),
        DynColors::Rgb(c.art_7.0, c.art_7.1, c.art_7.2),
        DynColors::Rgb(c.art_8.0, c.art_8.1, c.art_8.2),
        DynColors::Rgb(c.art_9.0, c.art_9.1, c.art_9.2),
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
