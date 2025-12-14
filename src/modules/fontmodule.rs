// Font finder module for Slowfetch.
// Parses terminal configs to find the in-use font.

use std::fs;
use std::env;
use memchr::memmem;
use super::userspacemodules::terminal;

// Get the terminal font by parsing config files
pub fn find_font() -> String {
    // Use the terminal detection from userspacemodules
    let term = terminal();

    // Try terminal-specific configs based on detected terminal
    let result = match term.to_lowercase().as_str() {
        "alacritty" => font_from_alacritty(),
        "kitty" => font_from_kitty(),
        "foot" => font_from_foot(),
        "ghostty" => font_from_ghostty(),
        "gnome terminal" => font_from_gnome_terminal(),
        "konsole" => font_from_konsole(),
        _ => None,
    };

    result.unwrap_or_else(|| "unknown".to_string())
}

// Parse Kitty config (~/.config/kitty/kitty.conf)
fn font_from_kitty() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let path = format!("{}/.config/kitty/kitty.conf", home);
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("font_family") && !line.starts_with('#') {
            // Format: font_family JetBrains Mono
            let font = line.trim_start_matches("font_family").trim();
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Alacritty config (~/.config/alacritty/alacritty.toml)
fn font_from_alacritty() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let path = format!("{}/.config/alacritty/alacritty.toml", home);
    let content = fs::read(&path).ok()?;

    // Find "family" using SIMD search
    let pos = memmem::find(&content, b"family")?;
    let after = &content[pos..];

    // Find '=' after family
    let eq_pos = memchr::memchr(b'=', after)?;
    let after_eq = &after[eq_pos + 1..];

    // Find the quoted value
    let quote1 = memchr::memchr(b'"', after_eq)?;
    let after_quote = &after_eq[quote1 + 1..];
    let quote2 = memchr::memchr(b'"', after_quote)?;

    let font = std::str::from_utf8(&after_quote[..quote2]).ok()?.trim();
    if !font.is_empty() {
        return Some(clean_font_name(font));
    }
    None
}

// Parse Foot config (~/.config/foot/foot.ini)
fn font_from_foot() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let path = format!("{}/.config/foot/foot.ini", home);
    let content = fs::read(&path).ok()?;

    // Find "font=" using SIMD search
    let pos = memmem::find(&content, b"\nfont=")?;
    let after = &content[pos + 6..]; // Skip "\nfont="

    // Find end of line or ':' (for size specifier)
    let end = memchr::memchr(b'\n', after).unwrap_or(after.len());
    let colon = memchr::memchr(b':', after).unwrap_or(end);
    let font_end = end.min(colon);

    let font = std::str::from_utf8(&after[..font_end]).ok()?.trim();
    if !font.is_empty() {
        return Some(clean_font_name(font));
    }
    None
}

// Parse Ghostty config (~/.config/ghostty/config)
fn font_from_ghostty() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let path = format!("{}/.config/ghostty/config", home);
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("font-family") && !line.starts_with('#') {
            // Format: font-family = JetBrains Mono
            let font = line
                .trim_start_matches("font-family")
                .trim()
                .trim_start_matches('=')
                .trim();
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Konsole profile (~/.local/share/konsole/*.profile)
fn font_from_konsole() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let profiles_dir = format!("{}/.local/share/konsole", home);

    let entries = fs::read_dir(&profiles_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "profile") {
            if let Ok(content) = fs::read(&path) {
                // Find "Font=" using SIMD search
                if let Some(pos) = memmem::find(&content, b"\nFont=") {
                    let after = &content[pos + 6..]; // Skip "\nFont="
                    // Find end at newline or comma
                    let end = memchr::memchr(b'\n', after).unwrap_or(after.len());
                    let comma = memchr::memchr(b',', after).unwrap_or(end);
                    let font_end = end.min(comma);

                    if let Ok(font) = std::str::from_utf8(&after[..font_end]) {
                        return Some(clean_font_name(font.trim()));
                    }
                }
            }
        }
    }
    Some("unset".to_string())
}

// Parse GNOME Terminal via dconf
fn font_from_gnome_terminal() -> Option<String> {
    // GNOME Terminal stores profile-specific fonts in dconf
    // First try to get the default profile's font
    let output = std::process::Command::new("dconf")
        .args(["dump", "/org/gnome/terminal/legacy/profiles:/"])
        .output()
        .ok()?;

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout);
        // Look for font= in any profile section
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("font=") {
                let font = line.trim_start_matches("font=").trim_matches('\'');
                // Format is "Font Name Size", strip the size
                let font = font.rsplit_once(' ').map(|(name, _)| name).unwrap_or(font);
                if !font.is_empty() {
                    return Some(clean_font_name(font));
                }
            }
        }
    }

    // Fallback: use system monospace font (what GNOME Terminal uses by default)
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "monospace-font-name"])
        .output()
        .ok()?;

    if output.status.success() {
        let font = String::from_utf8_lossy(&output.stdout);
        let font = font.trim().trim_matches('\'');
        // Format is "Font Name Size", strip the size
        let font = font.rsplit_once(' ').map(|(name, _)| name).unwrap_or(font);
        if !font.is_empty() {
            return Some(clean_font_name(font));
        }
    }

    None
}

// Check if a font name indicates if its a nerd font
pub fn is_nerd_font(font: &str) -> bool {
    // NF or Nerd Font, this isnt robust because people can set their fonts wrong but its safer than
    // non nerd users getting garbled outputs.
    font.contains("NF") || font.contains("Nerd Font")
}

// Clean up font name - remove style suffixes, normalize, and beautify for display
fn clean_font_name(font: &str) -> String {
    let font = font.trim();

    // Resolve generic font aliases like "monospace" using fc-match
    let font = resolve_font_alias(font);

    // Remove common style suffixes if they appear at the end (case-insensitive)
    let suffixes: &[&[u8]] = &[
        b" regular", b" medium", b" bold", b" italic", b" light",
        b" thin", b" semibold", b" extrabold", b" black",
    ];

    let mut result = font;
    let lower = result.to_lowercase();
    let lower_bytes = lower.as_bytes();
    for suffix in suffixes {
        if lower_bytes.ends_with(suffix) {
            result = result[..result.len() - suffix.len()].to_string();
            break;
        }
    }

    // Convert "Nerd Font" to "NF" using SIMD search
    if memmem::find(result.as_bytes(), b"Nerd Font").is_some() {
        result = result.replace("Nerd Font", "NF");
    }

    // Trim " Mono" from end
    if result.as_bytes().ends_with(b" Mono") {
        result = result[..result.len() - 5].to_string();
    }

    result
}

// Resolve generic font aliases (monospace, sans-serif, etc.) to actual font names
fn resolve_font_alias(font: &str) -> String {
    let generic_aliases = ["monospace", "sans-serif", "serif", "mono", "system-ui"];

    if generic_aliases.contains(&font.to_lowercase().as_str()) {
        // Use fc-match to resolve the alias
        if let Ok(output) = std::process::Command::new("fc-match")
            .arg(font)
            .arg("-f")
            .arg("%{family}")
            .output()
        {
            if output.status.success() {
                let resolved = String::from_utf8_lossy(&output.stdout);
                let resolved = resolved.trim();
                if !resolved.is_empty() {
                    return resolved.to_string();
                }
            }
        }
    }

    font.to_string()
}
