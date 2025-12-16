// Configuration loader for Slowfetch
// Loads settings from config.toml

use std::fs;
use std::path::PathBuf;
use memchr::{memchr, memchr_iter};

// Embed the default config file at compile time
const DEFAULT_CONFIG: &str = include_str!("config.toml");

// OS art setting - can be disabled, auto-detect, or specific OS
#[derive(Debug, Clone)]
pub enum OsArtSetting {
    Disabled,
    Auto,
    Specific(String),
}

// Theme color - can be RGB or ANSI (0-15)
#[derive(Debug, Clone, Copy)]
pub enum ThemeColor {
    Rgb(u8, u8, u8),
    Ansi(u8), // ANSI color code 0-15
}

// Color configuration - theme colors can be RGB or ANSI
// Art colors are always RGB tuples
#[derive(Debug, Clone)]
pub struct ColorConfig {
    // Theme colors
    pub border: ThemeColor,
    pub title: ThemeColor,
    pub key: ThemeColor,
    pub value: ThemeColor,
    // ASCII art colors (1-9)
    pub art_1: (u8, u8, u8),
    pub art_2: (u8, u8, u8),
    pub art_3: (u8, u8, u8),
    pub art_4: (u8, u8, u8),
    pub art_5: (u8, u8, u8),
    pub art_6: (u8, u8, u8),
    pub art_7: (u8, u8, u8),
    pub art_8: (u8, u8, u8),
    pub art_9: (u8, u8, u8),
}

// Theme presets for theme colors (border, title, key, value)
// Default uses None to indicate "use terminal's default colors"
#[derive(Debug, Clone, Copy, Default)]
pub enum ThemePreset {
    #[default]
    Default,
    Dracula,
    Catppuccin,
    Nord,
    Gruvbox,
    Eldritch,
    Kanagawa,
}

impl ThemePreset {
    // Parse theme name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" | "terminal" | "tty" => Some(Self::Default),
            "dracula" => Some(Self::Dracula),
            "catppuccin" | "cat" => Some(Self::Catppuccin),
            "nord" => Some(Self::Nord),
            "gruvbox" | "gruv" => Some(Self::Gruvbox),
            "eldritch" => Some(Self::Eldritch),
            "kanagawa" | "kana" => Some(Self::Kanagawa),
            _ => None,
        }
    }

    // Get theme colors: (border, title, key, value)
    pub fn colors(self) -> (ThemeColor, ThemeColor, ThemeColor, ThemeColor) {
        match self {
            // Default: ANSI 1 (red), 3 (yellow), 4 (blue), 5 (magenta)
            Self::Default => (
                ThemeColor::Ansi(4), // border: magenta
                ThemeColor::Ansi(5), // title:  pink
                ThemeColor::Ansi(5), // key:    pink
                ThemeColor::Ansi(6), // value:  light blue
            ),
            Self::Dracula => (
                ThemeColor::Rgb(0xFF, 0x79, 0xC6), // border: #FF79C6 - pink
                ThemeColor::Rgb(0xFF, 0x79, 0xC6), // title:  #FF79C6 - pink
                ThemeColor::Rgb(0xBD, 0x93, 0xF9), // key:    #BD93F9 - purple
                ThemeColor::Rgb(0x8B, 0xE9, 0xFD), // value:  #8BE9FD - cyan
            ),
            Self::Catppuccin => (
                // Catppuccin Mocha - https://catppuccin.com/palette
                ThemeColor::Rgb(0xF3, 0x8B, 0xA8), // border: #F5C2E7 - pinky red
                ThemeColor::Rgb(0xCB, 0xA6, 0xF7), // title:  #CBA6F7 - mauve
                ThemeColor::Rgb(0x89, 0xB4, 0xFA), // key:    #89B4FA - blue
                ThemeColor::Rgb(0xF5, 0xC2, 0xE7), // value:  #F38BA8 - pink
            ),
            Self::Nord => (
                // Nord - https://nordtheme.com/docs/colors-and-palettes
                ThemeColor::Rgb(0xB4, 0x8E, 0xAD), // border: #B48EAD - nord15 purple
                ThemeColor::Rgb(0x88, 0xC0, 0xD0), // title:  #88C0D0 - nord8 frost cyan
                ThemeColor::Rgb(0x81, 0xA1, 0xC1), // key:    #81A1C1 - nord9 frost blue
                ThemeColor::Rgb(0x5E, 0x81, 0xAC), // value:  #5E81AC - nord10 frost dark blue
            ),
            Self::Gruvbox => (
                // Gruvbox Dark - https://github.com/morhetz/gruvbox
                ThemeColor::Rgb(0xB8, 0xBB, 0x26), // border: #B8BB26 - bright green
                ThemeColor::Rgb(0xFB, 0x49, 0x34), // title:  #FB4934 - bright red
                ThemeColor::Rgb(0xFA, 0xBD, 0x2F), // key:    #FABD2F - bright yellow
                ThemeColor::Rgb(0x83, 0xA5, 0x98), // value:  #83A598 - bright blue
            ),
            Self::Eldritch => (
                // Eldritch - https://github.com/eldritch-theme/eldritch
                ThemeColor::Rgb(0xF2, 0x65, 0xB5), // border: #F265B5 - pink
                ThemeColor::Rgb(0xA4, 0x8C, 0xF2), // title:  #A48CF2 - purple
                ThemeColor::Rgb(0x37, 0xF4, 0x99), // key:    #37F499 - green
                ThemeColor::Rgb(0x04, 0xD1, 0xF9), // value:  #04D1F9 - cyan
            ),
            Self::Kanagawa => (
                // Kanagawa Wave - https://github.com/rebelot/kanagawa.nvim
                ThemeColor::Rgb(0xC0, 0xA3, 0x6E), // border: #76946A - boatYellow2
                ThemeColor::Rgb(0x76, 0x94, 0x6A), // title:  #C0A36E - autumnGreen
                ThemeColor::Rgb(0x7E, 0x9C, 0xD8), // key:    #7E9CD8 - crystalBlue
                ThemeColor::Rgb(0x98, 0xBB, 0x6C), // value:  #98BB6C - springGreen
            ),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        let (border, title, key, value) = ThemePreset::default().colors();
        Self {
            // Default theme colors (uses terminal defaults)
            border,
            title,
            key,
            value,
            // Default art colors (rainbow spectrum)
            art_1: (0xFF, 0x00, 0x00), // #FF0000 - Red
            art_2: (0xFF, 0x80, 0x00), // #FF8000 - Orange
            art_3: (0xFF, 0xFF, 0x00), // #FFFF00 - Yellow
            art_4: (0x00, 0xFF, 0x00), // #00FF00 - Green
            art_5: (0x00, 0xFF, 0xFF), // #00FFFF - Cyan
            art_6: (0x00, 0xBF, 0xFF), // #00BFFF - Light Blue
            art_7: (0x55, 0x55, 0xFF), // #5555FF - Blue
            art_8: (0xAA, 0x55, 0xFF), // #AA55FF - Violet
            art_9: (0xFF, 0x55, 0xFF), // #FF55FF - Magenta
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub os_art: OsArtSetting,
    pub colors: ColorConfig,
    pub custom_art: Option<String>,
    pub image: bool,
    pub image_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            os_art: OsArtSetting::Disabled,
            colors: ColorConfig::default(),
            custom_art: None,
            image: false,
            image_path: None,
        }
    }
}

// Parse a hex color string like "#FF79C6" or "FF79C6" into RGB tuple
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_matches('"');
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}

// Get the config directory path
fn get_config_dir() -> Option<PathBuf> {
    // Prefer XDG_CONFIG_HOME if set
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg_config).join("slowfetch"));
    }

    // Fall back to ~/.config/slowfetch
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(&home).join(".config/slowfetch"));
    }

    None
}

// Get the config file path, checking common locations
fn get_config_path() -> Option<PathBuf> {
    // Check XDG_CONFIG_HOME/slowfetch/config.toml first
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("slowfetch/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Check ~/.config/slowfetch/config.toml
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(&home).join(".config/slowfetch/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Check config.toml in current directory (for development)
    let local_path = PathBuf::from("config.toml");
    if local_path.exists() {
        return Some(local_path);
    }

    None
}

// Install the default config file to ~/.config/slowfetch/config.toml
fn install_default_config() -> Option<PathBuf> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        if fs::create_dir_all(&config_dir).is_err() {
            eprintln!(
                "Warning: Could not create config directory: {:?}",
                config_dir
            );
            return None;
        }
    }

    // Write the default config file
    if fs::write(&config_path, DEFAULT_CONFIG).is_err() {
        eprintln!("Warning: Could not write config file: {:?}", config_path);
        return None;
    }

    eprintln!("Installed default config to: {:?}", config_path);
    Some(config_path)
}

// Load configuration from file
pub fn load_config() -> Config {
    // Try to find an existing config file
    let path = match get_config_path() {
        Some(p) => p,
        None => {
            // No config found, install the default one
            match install_default_config() {
                Some(p) => p,
                None => return Config::default(),
            }
        }
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    parse_config(&content)
}

// Parse the TOML config content using byte-level operations
fn parse_config(content: &str) -> Config {
    let mut config = Config::default();
    let bytes = content.as_bytes();
    let mut in_colors_section = false;

    // First pass: look for theme preset (so individual colors can override it)
    let mut start = 0;
    for end in memchr_iter(b'\n', bytes).chain(std::iter::once(bytes.len())) {
        let line = &bytes[start..end];
        start = end + 1;

        if line.is_empty() {
            continue;
        }

        let first_nonws = match line.iter().position(|&b| b != b' ' && b != b'\t') {
            Some(p) => p,
            None => continue,
        };
        let line = &line[first_nonws..];

        if line.first() == Some(&b'#') || line.first() == Some(&b'[') {
            continue;
        }

        let eq_pos = match memchr(b'=', line) {
            Some(p) => p,
            None => continue,
        };

        let key = trim_bytes(&line[..eq_pos]);
        let value = trim_bytes(&line[eq_pos + 1..]);

        if key == b"theme" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(theme_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if let Some(preset) = ThemePreset::from_str(theme_name) {
                        let (border, title, key, val) = preset.colors();
                        config.colors.border = border;
                        config.colors.title = title;
                        config.colors.key = key;
                        config.colors.value = val;
                    }
                }
            }
            break;
        }
    }

    // Second pass: process all settings (individual colors override theme)
    start = 0;
    for end in memchr_iter(b'\n', bytes).chain(std::iter::once(bytes.len())) {
        let line = &bytes[start..end];
        start = end + 1;

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Find first non-whitespace
        let first_nonws = match line.iter().position(|&b| b != b' ' && b != b'\t') {
            Some(p) => p,
            None => continue,
        };
        let line = &line[first_nonws..];

        // Skip comments
        if line.first() == Some(&b'#') {
            continue;
        }

        // Track sections
        if line.first() == Some(&b'[') {
            in_colors_section = line == b"[colors]";
            continue;
        }

        // Find '=' for key=value parsing
        let eq_pos = match memchr(b'=', line) {
            Some(p) => p,
            None => continue,
        };

        let key = &line[..eq_pos];
        let value = &line[eq_pos + 1..];

        // Trim key and value
        let key = trim_bytes(key);
        let value = trim_bytes(value);

        // Parse color settings
        if in_colors_section {
            if let (Ok(key_str), Ok(value_str)) = (std::str::from_utf8(key), std::str::from_utf8(value)) {
                if let Some(color) = parse_hex_color(value_str) {
                    match key_str {
                        "border" => config.colors.border = ThemeColor::Rgb(color.0, color.1, color.2),
                        "title" => config.colors.title = ThemeColor::Rgb(color.0, color.1, color.2),
                        "key" => config.colors.key = ThemeColor::Rgb(color.0, color.1, color.2),
                        "value" => config.colors.value = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_1" => config.colors.art_1 = color,
                        "art_2" => config.colors.art_2 = color,
                        "art_3" => config.colors.art_3 = color,
                        "art_4" => config.colors.art_4 = color,
                        "art_5" => config.colors.art_5 = color,
                        "art_6" => config.colors.art_6 = color,
                        "art_7" => config.colors.art_7 = color,
                        "art_8" => config.colors.art_8 = color,
                        "art_9" => config.colors.art_9 = color,
                        _ => {}
                    }
                }
            }
            continue;
        }

        // Parse os_art setting
        if key == b"os_art" {
            if value == b"true" {
                config.os_art = OsArtSetting::Auto;
            } else if value == b"false" {
                config.os_art = OsArtSetting::Disabled;
            } else if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(os_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !os_name.is_empty() {
                        config.os_art = OsArtSetting::Specific(os_name.to_string());
                    }
                }
            }
        }

        // Parse custom_art setting
        if key == b"custom_art" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(path) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !path.is_empty() {
                        let expanded = expand_home_path(path);
                        config.custom_art = Some(expanded);
                    }
                }
            }
        }

        // Parse image toggle (but not image_path)
        if key == b"image" {
            config.image = value == b"true";
        }

        // Parse image_path setting
        if key == b"image_path" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(path) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !path.is_empty() {
                        let expanded = expand_home_path(path);
                        config.image_path = Some(expanded);
                    }
                }
            }
        }
    }

    config
}

// Trim leading and trailing whitespace from bytes
#[inline]
fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(bytes.len());
    let end = bytes.iter().rposition(|&b| b != b' ' && b != b'\t').map(|p| p + 1).unwrap_or(start);
    &bytes[start..end]
}

// Expand ~ to home directory
#[inline]
fn expand_home_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}