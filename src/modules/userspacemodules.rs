// Userspace/software/whatever information modules for Slowfetch

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use memchr::{memchr_iter, memmem};

use crate::helpers::{capitalize, get_cached_is_nerd_font, get_dms_theme, get_noctalia_scheme};

/// Get the active shell with version.
pub fn shell() -> String {
    let shell_path = match env::var("SHELL") {
        Ok(p) => p,
        Err(_) => return "unknown".to_string(),
    };

    let shell_name = match shell_path.rsplit('/').next() {
        Some(name) if !name.is_empty() => name,
        _ => return "unknown".to_string(),
    };

    // Try to get version by running shell --version
    let version = Command::new(&shell_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            // Find first line directly in bytes using memchr
            let stdout = &output.stdout;
            let first_line_end = memchr::memchr(b'\n', stdout).unwrap_or(stdout.len());
            let first_line = std::str::from_utf8(&stdout[..first_line_end]).ok()?;

            // Extract version number (e.g., "5.2.26" from "bash 5.2.26(1)-release")
            first_line
                .split_ascii_whitespace()
                .find(|word| word.as_bytes().first().map_or(false, |b| b.is_ascii_digit()))
                .map(|v| {
                    // Clean up version string - find first ( or - using memchr
                    let v_bytes = v.as_bytes();
                    let paren_pos = memchr::memchr(b'(', v_bytes);
                    let dash_pos = memchr::memchr(b'-', v_bytes);
                    let end = match (paren_pos, dash_pos) {
                        (Some(p), Some(d)) => p.min(d),
                        (Some(p), None) => p,
                        (None, Some(d)) => d,
                        (None, None) => v.len(),
                    };
                    v[..end].to_string()
                })
        });

    match version {
        Some(v) => format!("{} {}", capitalize(shell_name), v),
        None => capitalize(shell_name),
    }
}

// Count RPM packages by querying rpmdb.sqlite directly via dlopen'd libsqlite3.
// This avoids spawning `rpm -qa` which takes ~600ms on systems with many packages.
// Uses the Sigmd5 table which naturally excludes gpg-pubkey virtual packages.
fn count_rpm_sqlite(db_path: &str) -> Option<usize> {
    use std::os::raw::{c_char, c_int, c_void};

    const SQLITE_OK: c_int = 0;
    const SQLITE_ROW: c_int = 100;
    const SQLITE_OPEN_READONLY: c_int = 0x00000001;

    // dlopen libsqlite3 - always present on RPM systems since rpm depends on it
    let lib_name = b"libsqlite3.so.0\0";
    let lib = unsafe { libc::dlopen(lib_name.as_ptr() as *const c_char, libc::RTLD_LAZY) };
    if lib.is_null() {
        return None;
    }

    // Load the 6 function pointers we need
    macro_rules! load_sym {
        ($lib:expr, $name:literal) => {{
            let sym = unsafe { libc::dlsym($lib, concat!($name, "\0").as_ptr() as *const c_char) };
            if sym.is_null() {
                unsafe { libc::dlclose($lib); }
                return None;
            }
            sym
        }};
    }

    let open_v2 = load_sym!(lib, "sqlite3_open_v2");
    let prepare_v2 = load_sym!(lib, "sqlite3_prepare_v2");
    let step = load_sym!(lib, "sqlite3_step");
    let column_int = load_sym!(lib, "sqlite3_column_int");
    let finalize = load_sym!(lib, "sqlite3_finalize");
    let close = load_sym!(lib, "sqlite3_close");

    type OpenV2Fn = unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int, *const c_char) -> c_int;
    type PrepareV2Fn = unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut *mut c_void, *mut *const c_char) -> c_int;
    type StepFn = unsafe extern "C" fn(*mut c_void) -> c_int;
    type ColumnIntFn = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
    type FinalizeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
    type CloseFn = unsafe extern "C" fn(*mut c_void) -> c_int;

    let sqlite3_open_v2: OpenV2Fn = unsafe { std::mem::transmute(open_v2) };
    let sqlite3_prepare_v2: PrepareV2Fn = unsafe { std::mem::transmute(prepare_v2) };
    let sqlite3_step: StepFn = unsafe { std::mem::transmute(step) };
    let sqlite3_column_int: ColumnIntFn = unsafe { std::mem::transmute(column_int) };
    let sqlite3_finalize: FinalizeFn = unsafe { std::mem::transmute(finalize) };
    let sqlite3_close: CloseFn = unsafe { std::mem::transmute(close) };

    // Build null-terminated path
    let mut path_buf = db_path.as_bytes().to_vec();
    path_buf.push(0);

    let mut db: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { sqlite3_open_v2(path_buf.as_ptr() as *const c_char, &mut db, SQLITE_OPEN_READONLY, std::ptr::null()) };
    if rc != SQLITE_OK {
        unsafe { libc::dlclose(lib); }
        return None;
    }

    let sql = b"SELECT count(*) FROM Sigmd5\0";
    let mut stmt: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { sqlite3_prepare_v2(db, sql.as_ptr() as *const c_char, -1, &mut stmt, std::ptr::null_mut()) };
    if rc != SQLITE_OK {
        unsafe { sqlite3_close(db); libc::dlclose(lib); }
        return None;
    }

    let count = if unsafe { sqlite3_step(stmt) } == SQLITE_ROW {
        Some(unsafe { sqlite3_column_int(stmt, 0) } as usize)
    } else {
        None
    };

    unsafe {
        sqlite3_finalize(stmt);
        sqlite3_close(db);
        libc::dlclose(lib);
    }

    count
}

// Get the total number of installed packages.
// Supports pacman aka Arch, hopefully supports debian and fedora but idk, im not setting up a vm to test sorry
pub fn packages() -> String {
    let mut counts: Vec<String> = Vec::with_capacity(4);
    let nerd = get_cached_is_nerd_font();

    // Pacman - count directories in /var/lib/pacman/local/
    if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
        let count = entries.filter(|e| e.is_ok()).count();
        if count > 0 {
            let icon = if nerd { "󰮯" } else { "(pacman)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // dpkg (Debian/Ubuntu) - count occurrences of status line using SIMD-accelerated search
    if let Ok(content) = fs::read("/var/lib/dpkg/status") {
        const NEEDLE: &[u8] = b"\nStatus: install ok installed\n";
        let count = memmem::find_iter(&content, NEEDLE).count();
        if count > 0 {
            let icon = if nerd { "󰕈" } else { "(dpkg)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // RPM - query rpmdb.sqlite directly via dlopen'd libsqlite3 (avoids spawning rpm -qa which is ~600ms)
    // Sigmd5 table naturally excludes gpg-pubkey virtual packages
    {
        let rpm_db_path = [
            "/usr/lib/sysimage/rpm/rpmdb.sqlite",
            "/var/lib/rpm/rpmdb.sqlite",
        ]
        .iter()
        .find(|p| Path::new(p).exists());

        if let Some(&db_path) = rpm_db_path {
            let count = count_rpm_sqlite(db_path).unwrap_or_else(|| {
                // Fallback to rpm -qa if sqlite query fails
                Command::new("rpm")
                    .arg("-qa")
                    .output()
                    .ok()
                    .map(|output| memchr_iter(b'\n', &output.stdout).count())
                    .unwrap_or(0)
            });
            if count > 0 {
                let icon = if nerd { "" } else { "(rpm)" };
                counts.push(format!("{} {}", icon, count));
            }
        } else if Path::new("/var/lib/rpm/Packages").exists() {
            // Legacy BDB format - must use rpm command
            if let Ok(output) = Command::new("rpm").arg("-qa").output() {
                let count = memchr_iter(b'\n', &output.stdout).count();
                if count > 0 {
                    let icon = if nerd { "" } else { "(rpm)" };
                    counts.push(format!("{} {}", icon, count));
                }
            }
        }
    }

    // Flatpak - count installed applications
    if let Ok(entries) = fs::read_dir("/var/lib/flatpak/app") {
        let count = entries.filter(|e| e.is_ok()).count();
        if count > 0 {
            let icon = if nerd { " " } else { "(flatpak)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // Nix - count packages in user profile
    if let Ok(home) = env::var("HOME") {
        let nix_profile = format!("{}/.nix-profile/manifest.nix", home);
        if Path::new(&nix_profile).exists() {
            // Count packages via nix-env -q
            if let Ok(output) = Command::new("nix-env").arg("-q").output() {
                // Count non-empty lines using SIMD-accelerated memchr
                let stdout = &output.stdout;
                let newline_count = memchr_iter(b'\n', stdout).count();
                // If output ends with newline, count equals lines; otherwise add 1 for last line
                let count = if stdout.last() == Some(&b'\n') || stdout.is_empty() {
                    newline_count
                } else {
                    newline_count + 1
                };
                if count > 0 {
                    let icon = if nerd { "󱄅" } else { "(nix)" };
                    counts.push(format!("{} {}", icon, count));
                }
            }
        }
    }

    // XBPS (Void Linux) - count directories in /var/db/xbps/
    if let Ok(entries) = fs::read_dir("/var/db/xbps") {
        let count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |ft| ft.is_dir()))
            .count();
        if count > 0 {
            let icon = if nerd { "" } else { "(xbps)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // Portage (Gentoo) - count package directories in /var/db/pkg/
    // Structure is /var/db/pkg/<category>/<package>-<version>/
    if let Ok(categories) = fs::read_dir("/var/db/pkg") {
        let count: usize = categories
            .filter_map(|cat| cat.ok())
            .filter(|cat| cat.file_type().map_or(false, |ft| ft.is_dir()))
            .filter_map(|cat| fs::read_dir(cat.path()).ok())
            .map(|pkgs| {
                pkgs.filter_map(|p| p.ok())
                    .filter(|p| p.file_type().map_or(false, |ft| ft.is_dir()))
                    .count()
            })
            .sum();
        if count > 0 {
            let icon = if nerd { "" } else { "(portage)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    if counts.is_empty() {
        "unknown".to_string()
    } else {
        counts.join(" | ")
    }
}

// Get the Window Manager (using /proc instead of subprocess)
pub fn wm() -> String {
    // Check environment variables first - much faster than /proc scan
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        // Map common desktop values to their WM names
        let wm = match desktop.to_lowercase().as_str() {
            "hyprland" => "Hyprland",
            "sway" => "Sway",
            "kde" | "plasma" => "KWin",
            "gnome" => "Mutter",
            "xfce" => "Xfwm4",
            "i3" => "i3",
            "bspwm" => "bspwm",
            "awesome" => "Awesome",
            "qtile" => "Qtile",
            "niri" => "Niri",
            _ => return desktop,
        };
        return wm.to_string();
    }

    if let Ok(session) = env::var("DESKTOP_SESSION") {
        return capitalize(&session);
    }

    // Fallback: scan /proc for WM processes
    // Known WMs to search for (search term -> display name)
    // Pre-compiled searchers for SIMD-accelerated matching
    let wm_list: &[(&[u8], &str)] = &[
        (b"mutter", "Mutter"),
        (b"kwin", "KWin"),
        (b"sway", "Sway"),
        (b"hyprland", "Hyprland"),
        (b"Hyprland", "Hyprland"),
        (b"river", "River"),
        (b"wayfire", "Wayfire"),
        (b"labwc", "LabWC"),
        (b"dwl", "dwl"),
        (b"niri", "Niri"),
        (b"openbox", "Openbox"),
        (b"i3", "i3"),
        (b"bspwm", "bspwm"),
        (b"dwm", "dwm"),
        (b"awesome", "Awesome"),
        (b"xfwm4", "Xfwm4"),
        (b"marco", "Marco"),
        (b"metacity", "Metacity"),
        (b"compiz", "Compiz"),
        (b"enlightenment", "Enlightenment"),
        (b"fluxbox", "Fluxbox"),
        (b"icewm", "IceWM"),
        (b"xmonad", "XMonad"),
        (b"qtile", "Qtile"),
        (b"herbstluftwm", "herbstluftwm"),
        (b"weston", "Weston"),
        (b"cage", "Cage"),
        (b"gamescope", "Gamescope"),
    ];

    // Read /proc directly instead of spawning ps | grep (saves 0.3ish ms)
    let proc_path = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            // Fast check: first byte must be a digit (PID directories)
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            if name_bytes.is_empty() || !name_bytes[0].is_ascii_digit() {
                continue;
            }

            let cmdline_path = entry.path().join("cmdline");
            // Read as bytes to avoid UTF-8 conversion overhead
            if let Ok(cmdline) = fs::read(&cmdline_path) {
                for (wm_search, wm_display) in wm_list {
                    if memmem::find(&cmdline, wm_search).is_some() {
                        return wm_display.to_string();
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

// Get the active terminal
pub fn terminal() -> String {
    // Check for specific terminal environment variables first
    if env::var("KITTY_PID").is_ok() {
        return "Kitty".to_string();
    }
    if env::var("KONSOLE_VERSION").is_ok() {
        return "Konsole".to_string();
    }
    if env::var("GNOME_TERMINAL_SCREEN").is_ok() {
        return "Gnome Terminal".to_string();
    }

    // Fallback to TERM_PROGRAM or TERM
    let term = env::var("TERM_PROGRAM")
        .unwrap_or_else(|_| env::var("TERM").unwrap_or_else(|_| "unknown".to_string()));

    // Clean up common suffixes like -256color
    let name = term.split("-256color").next().unwrap_or(&term);
    let name = name.split("-color").next().unwrap_or(name);

    capitalize(name)
}

// Get the active UI/Shell, i dont know what to call this shit because i already used shell for the terminal shell
pub fn ui() -> String {
    // Scan /proc for custom shells first (noctalia, dms) - these take priority over env vars
    let proc_path = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            // Fast check: first byte must be a digit (PID directories)
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            if name_bytes.is_empty() || !name_bytes[0].is_ascii_digit() {
                continue;
            }

            let cmdline_path = entry.path().join("cmdline");
            // Read as bytes to avoid UTF-8 conversion overhead
            if let Ok(cmdline) = fs::read(&cmdline_path) {
                if memmem::find(&cmdline, b"noctalia-shell").is_some() {
                    let mut name = "Noctalia Shell".to_string();
                    if let Some(scheme) = get_noctalia_scheme() {
                        let icon = if get_cached_is_nerd_font() { "" } else { "Theme:" };
                        name = format!("{} | {} {}", name, icon, capitalize(&scheme));
                    }
                    return name;
                }
                if memmem::find(&cmdline, b"dms").is_some() {
                    let mut name = "DMS".to_string();
                    if let Some(theme) = get_dms_theme() {
                        let formatted_theme = theme
                            .replace("cat-", "Catppuccin (")
                            + if theme.starts_with("cat-") { ")" } else { "" };
                        let icon = if get_cached_is_nerd_font() { " " } else { "Theme:" };
                        name = format!("{} | {} {}", name, icon, capitalize(&formatted_theme));
                    }
                    return name;
                }
                //i know this janky but idk, its a fallback
                if memmem::find(&cmdline, b"plasmashell").is_some() {
                    return "Plasma Shell".to_string();
                }
                if memmem::find(&cmdline, b"gnome-shell").is_some() {
                    return "Gnome Shell".to_string();
                }
                if memmem::find(&cmdline, b"waybar").is_some() {
                    return "Custom Waybar setup".to_string();
                }
            }
        }
    }

    // Fallback: check env vars for common desktop shells
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        match desktop.to_lowercase().as_str() {
            "kde" | "plasma" => return "Plasma Shell".to_string(),
            "gnome" => return "Gnome Shell".to_string(),
            _ => {}
        }
    }

    "unknown".to_string()
}

// Get the user's preferred editor from environment variables.
// Returns empty string if unset or set to nano (dont @ me)
pub fn editor() -> String {
    let visual = env::var("VISUAL").ok();
    let editor = env::var("EDITOR").ok();

    // Helper to extract and format editor name
    let format_editor = |path: &str| -> Option<String> {
        let name = path.split('/').last().unwrap_or(path);
        if name == "nano" {
            None
        } else {
            Some(capitalize(name))
        }
    };

    match (visual.as_deref().and_then(format_editor), editor.as_deref().and_then(format_editor)) {
        (Some(v), Some(e)) if v != e => {
            let (icon1, icon2) = if get_cached_is_nerd_font() { ("󰍹", "") } else { ("GUI:", "TUI:") };
            format!("{} {} | {} {}", icon1, v, icon2, e)
        }
        (Some(v), _) => v,
        (None, Some(e)) => e,
        (None, None) => String::new()
    }
}
