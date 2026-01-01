// Preview generation for prettyconfig TUI
// Handles live preview updates and section filtering

use crate::configloader::{Config, CoreToggles, HardwareToggles, ThemePreset, UserspaceToggles};
use crate::prettyconfig::navigation::App;
use crate::visuals::colorcontrol;
use crate::visuals::renderer::{self, Section};

// Update the preview based on current app state
pub fn update_preview(app: &mut App) {
    // Set preview colors based on current theme
    let (border, title, key, value) = app.theme.colors();
    let preview_colors = crate::configloader::ColorConfig {
        border,
        title,
        key,
        value,
        ..crate::configloader::ColorConfig::default()
    };
    colorcontrol::set_preview_colors(Some(preview_colors));

    // Filter sections based on toggles
    let core = filter_core_section(&app.cached_sections.0, &app.core);
    let hardware = filter_hardware_section(&app.cached_sections.1, &app.hardware);
    let userspace = filter_userspace_section(&app.cached_sections.2, &app.userspace);

    let sections: Vec<_> = [core, hardware, userspace]
        .into_iter()
        .filter(|s| !s.lines.is_empty())
        .collect();

    // Generate full preview (art + sections)
    let (wide, narrow, smol) = app.get_art_for_preview();
    let output = renderer::draw_layout(&wide, &narrow, &sections, smol.as_deref());
    app.preview_lines = output.lines().map(String::from).collect();

    // Generate sections-only preview for image mode
    app.sections_only_lines = renderer::build_sections_lines(&sections, None);
}

// Detect theme from config colors
pub fn detect_theme_from_config(config: &Config) -> ThemePreset {
    let presets = [
        ThemePreset::Default,
        ThemePreset::Dracula,
        ThemePreset::Catppuccin,
        ThemePreset::Nord,
        ThemePreset::Gruvbox,
        ThemePreset::Eldritch,
        ThemePreset::Kanagawa,
    ];

    for preset in presets {
        let (border, _, _, _) = preset.colors();
        if std::mem::discriminant(&border) == std::mem::discriminant(&config.colors.border) {
            match (border, config.colors.border) {
                (crate::configloader::ThemeColor::Rgb(r1, g1, b1), crate::configloader::ThemeColor::Rgb(r2, g2, b2)) => {
                    if r1 == r2 && g1 == g2 && b1 == b2 {
                        return preset;
                    }
                }
                (crate::configloader::ThemeColor::Ansi(c1), crate::configloader::ThemeColor::Ansi(c2)) => {
                    if c1 == c2 {
                        return preset;
                    }
                }
                _ => {}
            }
        }
    }

    ThemePreset::Default
}

// Filter core section based on toggles
fn filter_core_section(section: &Section, toggles: &CoreToggles) -> Section {
    let lines: Vec<_> = section.lines.iter()
        .filter(|(key, _)| match key.as_str() {
            "OS" => toggles.os,
            "Kernel" => toggles.kernel,
            "Uptime" => toggles.uptime,
            "Init" => toggles.init,
            _ => true,
        })
        .cloned()
        .collect();
    Section::new(&section.title, lines)
}

// Filter hardware section based on toggles
fn filter_hardware_section(section: &Section, toggles: &HardwareToggles) -> Section {
    let lines: Vec<_> = section.lines.iter()
        .filter(|(key, _)| match key.as_str() {
            "CPU" => toggles.cpu,
            "GPU" => toggles.gpu,
            "Memory" => toggles.memory,
            "Storage" => toggles.storage,
            "Battery" => toggles.battery,
            k if k.starts_with("Display") || k.starts_with("├") || k.starts_with("╰") => toggles.screen,
            _ => true,
        })
        .cloned()
        .collect();
    Section::new(&section.title, lines)
}

// Filter userspace section based on toggles
fn filter_userspace_section(section: &Section, toggles: &UserspaceToggles) -> Section {
    let lines: Vec<_> = section.lines.iter()
        .filter(|(key, _)| match key.as_str() {
            "Packages" => toggles.packages,
            "Terminal" => toggles.terminal,
            "Shell" => toggles.shell,
            "WM" => toggles.wm,
            "UI" => toggles.ui,
            "Editor" => toggles.editor,
            "Terminal Font" => toggles.terminal_font,
            _ => true,
        })
        .cloned()
        .collect();
    Section::new(&section.title, lines)
}
