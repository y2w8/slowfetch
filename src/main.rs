//Slowfetch by Tūī

mod cache;
mod configloader;
mod dostuff;
mod helpers;
mod modules;
mod visuals;

use clap::Parser;
use configloader::OsArtSetting;

// cmd line args, *claps*
#[derive(Parser)]
#[command(name = "slowfetch", about = "A slow system info fetcher")]
struct Args {
    // Display OS-specific art. Optionally specify OS name (example: --os arch)
    #[arg(short = 'o', long = "os", num_args = 0..=1, default_missing_value = "")]
    os_art: Option<String>,

    // Force refresh of cached values (OS name and GPU)
    #[arg(short = 'r', long = "refresh")]
    refresh: bool,

    // Display image instead of ASCII art (uses Kitty graphics protocol)
    #[arg(short = 'i', long = "image", num_args = 0..=1, default_missing_value = "")]
    image: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Set cache refresh flag if --refresh/-r was passed
    if args.refresh {
        cache::set_force_refresh(true);
    }

    // Load config first and initialize colors before spawning threads
    let config = configloader::load_config();
    visuals::colorcontrol::init_colors(config.colors.clone());

    // Load all sections based on config
    let (core, hardware, userspace) = dostuff::load_sections(&config);

    // Extract OS name before filtering (needed for OS art detection)
    let os_name: String = core
        .lines
        .iter()
        .find(|(k, _)| k == "OS")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    // Filter out empty sections
    let sections: Vec<_> = [core, hardware, userspace]
        .into_iter()
        .filter(|s| !s.lines.is_empty())
        .collect();

    // Load ASCII art synchronously - just reading static data
    let wide_logo = modules::asciimodule::get_wide_logo_lines();
    let narrow_logo = modules::asciimodule::get_narrow_logo_lines();

    // Check if image mode is requested (CLI arg or config) AND terminal supports it
    let use_image = args.image.is_some() || config.image;

    if use_image {
        // Determine image path:
        // 1. CLI arg with explicit path takes highest priority
        // 2. CLI arg empty (-i/--image) uses config.image_path if set, else embedded default
        // 3. Config image=true uses config.image_path if set, else embedded default
        let image_path: Option<std::path::PathBuf> = if let Some(ref image_arg) = args.image {
            if image_arg.is_empty() {
                // CLI flag without path - use config image_path if available
                config.image_path.as_ref().map(std::path::PathBuf::from)
            } else if image_arg.starts_with("~/") {
                // CLI flag with explicit path (expand ~)
                Some(if let Some(home) = std::env::var_os("HOME") {
                    std::path::PathBuf::from(home).join(&image_arg[2..])
                } else {
                    std::path::PathBuf::from(image_arg)
                })
            } else {
                // CLI flag with explicit path
                Some(std::path::PathBuf::from(image_arg))
            }
        } else {
            // Config image=true, use config image_path if set
            config.image_path.as_ref().map(std::path::PathBuf::from)
        };

        // Draw image layout (imagerender handles all the logic)
        visuals::imagerender::draw_image_layout(&sections, image_path.as_deref());
    } else {
        // Standard ASCII art mode
        // Check for custom art first (overrides everything else)
        let (wide, narrow, smol) = if let Some(ref custom_path) = config.custom_art {
            if let Some(custom_art) = modules::asciimodule::get_custom_art_lines(custom_path) {
                (custom_art.clone(), custom_art, None)
            } else {
                // Custom art file not found, fall back to default
                (wide_logo.clone(), narrow_logo.clone(), None)
            }
        } else {
            // Determine OS art setting: CLI args override config
            let os_art_setting = if let Some(ref os_override) = args.os_art {
                if os_override.is_empty() {
                    OsArtSetting::Auto
                } else {
                    OsArtSetting::Specific(os_override.clone())
                }
            } else {
                config.os_art.clone()
            };

            // Apply OS art setting
            match os_art_setting {
                OsArtSetting::Disabled => (wide_logo, narrow_logo, None),
                OsArtSetting::Auto => {
                    if let Some(os_logo) = modules::asciimodule::get_os_logo_lines(&os_name) {
                        let smol_logo = modules::asciimodule::get_os_logo_lines_smol(&os_name);
                        (os_logo.clone(), os_logo, smol_logo)
                    } else {
                        (wide_logo, narrow_logo, None)
                    }
                }
                OsArtSetting::Specific(ref specific_os) => {
                    if let Some(os_logo) = modules::asciimodule::get_os_logo_lines(specific_os) {
                        let smol_logo = modules::asciimodule::get_os_logo_lines_smol(specific_os);
                        (os_logo.clone(), os_logo, smol_logo)
                    } else {
                        (wide_logo, narrow_logo, None)
                    }
                }
            }
        };

        print!(
            "{}",
            visuals::renderer::draw_layout(&wide, &narrow, &sections, smol.as_deref())
        );
    }
}
