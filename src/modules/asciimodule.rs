// ASCII art module for Slowfetch
// Renders colorized ASCII art using placeholder-based colorization

use crate::visuals::colorcontrol::get_art_colors;
use crate::visuals::asciiengine::AsciiArtColorizer;
use std::fs;

// The ASCII art for the Slowfetch logo Wide version.
const ASCII_ART_WIDE: &str = include_str!("../assets/default/wide.txt");

// The ASCII art for the Slowfetch logo narrow version.
const ASCII_ART_NARROW: &str = include_str!("../assets/default/narrow.txt");

// OS-specific ASCII art
const ASCII_ART_ARCH: &str = include_str!("../assets/arch.txt");
const ASCII_ART_CACHYOS: &str = include_str!("../assets/cachy.txt");
const ASCII_ART_FEDORA: &str = include_str!("../assets/fedora.txt");
const ASCII_ART_UBUNTU: &str = include_str!("../assets/ubuntu.txt");
const ASCII_ART_NIX: &str = include_str!("../assets/nix.txt");
const ASCII_ART_GENTOO: &str = include_str!("../assets/gentoo.txt");
const ASCII_ART_VOID: &str = include_str!("../assets/void.txt");
const ASCII_ART_PIKA: &str = include_str!("../assets/pika.txt");

// Meme versions
const ASCII_ART_ARCHMEME: &str = include_str!("../assets/archmeme.txt");
const ASCII_ART_NIXMEME: &str = include_str!("../assets/nixmeme.txt");

// Smol versions of OS-specific ASCII art
const ASCII_ART_ARCH_SMOL: &str = include_str!("../assets/archsmol.txt");
const ASCII_ART_CACHYOS_SMOL: &str = include_str!("../assets/cachysmol.txt");
const ASCII_ART_FEDORA_SMOL: &str = include_str!("../assets/fedorasmol.txt");
const ASCII_ART_UBUNTU_SMOL: &str = include_str!("../assets/ubuntusmol.txt");
const ASCII_ART_NIX_SMOL: &str = include_str!("../assets/nixsmol.txt");
const ASCII_ART_GENTOO_SMOL: &str = include_str!("../assets/gentoosmol.txt");
const ASCII_ART_VOID_SMOL: &str = include_str!("../assets/voidsmol.txt");
const ASCII_ART_PIKA_SMOL: &str = include_str!("../assets/pikasmol.txt");

// Render the wide ASCII art logo and return lines as a Vec
pub fn get_wide_logo_lines() -> Vec<String> {
    let colors = get_art_colors();
    AsciiArtColorizer::with_colors(ASCII_ART_WIDE, &colors, true).collect()
}

// Render the narrow ASCII art logo and return lines as a Vec
pub fn get_narrow_logo_lines() -> Vec<String> {
    let colors = get_art_colors();
    AsciiArtColorizer::with_colors(ASCII_ART_NARROW, &colors, true).collect()
}

// Get OS-specific art if available, returns None if no match
pub fn get_os_logo_lines(os_name: &str) -> Option<Vec<String>> {
    let os_lower = os_name.to_lowercase();
    // Check for meme versions first (explicit override only)
    let art_str = if os_lower == "archbtw" {
        Some(ASCII_ART_ARCHMEME)
    } else if os_lower == "nixbtw" {
        Some(ASCII_ART_NIXMEME)
    // Regular OS matching
    } else if os_lower.contains("cachyos") || os_lower.contains("cachy") {
        Some(ASCII_ART_CACHYOS)
    } else if os_lower.contains("arch") {
        Some(ASCII_ART_ARCH)
    } else if os_lower.contains("fedora") {
        Some(ASCII_ART_FEDORA)
    } else if os_lower.contains("ubuntu") {
        Some(ASCII_ART_UBUNTU)
    } else if os_lower.contains("nixos") || os_lower.contains("nix") {
        Some(ASCII_ART_NIX)
    } else if os_lower.contains("gentoo") {
        Some(ASCII_ART_GENTOO)
    } else if os_lower.contains("void") {
        Some(ASCII_ART_VOID)
    } else if os_lower.contains("pika") {
        Some(ASCII_ART_PIKA)
    } else {
        None
    };

    art_str.map(|s| {
        let colors = get_art_colors();
        AsciiArtColorizer::with_colors(s, &colors, true).collect()
    })
}

// Get smol OS-specific art if available, returns None if no match
pub fn get_os_logo_lines_smol(os_name: &str) -> Option<Vec<String>> {
    let os_lower = os_name.to_lowercase();
    // Check for special versions first (explicit override only)
    let art_str = if os_lower == "archbtw" {
        None // No smol version for meme
    } else if os_lower == "nixbtw" {
        None // No smol version for meme
    // Regular OS matching
    } else if os_lower.contains("cachyos") || os_lower.contains("cachy") {
        Some(ASCII_ART_CACHYOS_SMOL)
    } else if os_lower.contains("arch") {
        Some(ASCII_ART_ARCH_SMOL)
    } else if os_lower.contains("fedora") {
        Some(ASCII_ART_FEDORA_SMOL)
    } else if os_lower.contains("ubuntu") {
        Some(ASCII_ART_UBUNTU_SMOL)
    } else if os_lower.contains("nixos") || os_lower.contains("nix") {
        Some(ASCII_ART_NIX_SMOL)
    } else if os_lower.contains("gentoo") {
        Some(ASCII_ART_GENTOO_SMOL)
    } else if os_lower.contains("void") {
        Some(ASCII_ART_VOID_SMOL)
    } else if os_lower.contains("pika") {
        Some(ASCII_ART_PIKA_SMOL)
    } else {
        None
    };

    art_str.map(|s| {
        let colors = get_art_colors();
        AsciiArtColorizer::with_colors(s, &colors, true).collect()
    })
}

// Load custom ASCII art from a file path
// Returns None if file doesn't exist or can't be read
pub fn get_custom_art_lines(path: &str) -> Option<Vec<String>> {
    let content = fs::read_to_string(path).ok()?;
    let colors = get_art_colors();
    Some(AsciiArtColorizer::with_colors(&content, &colors, true).collect())
}
