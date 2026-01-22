// Hardware information modules for Slowfetch.
// Contains functions hardware, what else did you expect idiot

use std::fs;
use std::process::Command;

use memchr::memchr_iter;
use memchr::memmem;

use crate::cache;
use crate::helpers::{create_bar, get_cached_is_nerd_font, is_laptop, read_first_line};

// Get the CPU model name with boost clock.
// Uses persistent cache to avoid repeated /proc reads.
pub fn cpu() -> String {
    // Check cache first (unless --refresh was passed)
    if let Some(cached) = cache::get_cached_cpu() {
        return cached;
    }

    // No cache hit, fetch fresh value
    let result = cpu_fresh();

    // Cache the result for next time
    cache::cache_cpu(&result);

    result
}

// Fetch CPU info fresh (no cache)
// Uses byte-level parsing with memchr for speed
fn cpu_fresh() -> String {
    let model = fs::read("/proc/cpuinfo").ok().and_then(|content| {
        // Find "model name" using SIMD search
        let needle = b"model name";
        let pos = memmem::find(&content, needle)?;
        let after_needle = &content[pos + needle.len()..];

        // Find the ':' separator
        let colon_pos = memchr::memchr(b':', after_needle)?;
        let after_colon = &after_needle[colon_pos + 1..];

        // Find end of line
        let line_end = memchr::memchr(b'\n', after_colon).unwrap_or(after_colon.len());
        let name_bytes = &after_colon[..line_end];

        // Convert to string and process
        let name = std::str::from_utf8(name_bytes).ok()?;
        let words: Vec<&str> = name.split_whitespace().collect();

        // Find where GPU info starts (e.g., "with Radeon Graphics", "w/ Intel UHD")
        let gpu_start = words
            .iter()
            .position(|&w| w.eq_ignore_ascii_case("with") || w.eq_ignore_ascii_case("w/"));
        let words = match gpu_start {
            Some(idx) => &words[..idx],
            None => &words[..],
        };

        Some(
            words
                .iter()
                .filter(|&&w| !w.ends_with("-Core") && w != "Processor")
                .copied()
                .collect::<Vec<_>>()
                .join(" "),
        )
    });

    let model = match model {
        Some(m) => m,
        None => return "unknown".to_string(),
    };

    // Get boost clock from cpufreq (in kHz)
    let boost_clock = read_first_line("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .and_then(|khz_str| khz_str.parse::<u64>().ok())
        .map(|khz| {
            let ghz = khz as f64 / 1_000_000.0;
            format!(" @ {:.2}GHz", ghz)
        })
        .unwrap_or_default();

    format!("{}{}", model, boost_clock)
}

// Get memory usage as a visual bar, 10 blocks = 100% usage
// Uses byte-level parsing with memchr for speed
pub fn memory() -> String {
    let mut total: u64 = 0;
    let mut available: u64 = 0;

    if let Ok(content) = fs::read("/proc/meminfo") {
        // Find MemTotal using SIMD search
        if let Some(pos) = memmem::find(&content, b"MemTotal:") {
            let after = &content[pos + 9..]; // "MemTotal:" is 9 bytes
            // Skip whitespace and find the number
            if let Some(start) = after.iter().position(|&b| b.is_ascii_digit()) {
                let num_start = &after[start..];
                let end = num_start.iter().position(|&b| !b.is_ascii_digit()).unwrap_or(num_start.len());
                if let Ok(s) = std::str::from_utf8(&num_start[..end]) {
                    total = s.parse().unwrap_or(0);
                }
            }
        }

        // Find MemAvailable using SIMD search
        if let Some(pos) = memmem::find(&content, b"MemAvailable:") {
            let after = &content[pos + 13..]; // "MemAvailable:" is 13 bytes
            if let Some(start) = after.iter().position(|&b| b.is_ascii_digit()) {
                let num_start = &after[start..];
                let end = num_start.iter().position(|&b| !b.is_ascii_digit()).unwrap_or(num_start.len());
                if let Ok(s) = std::str::from_utf8(&num_start[..end]) {
                    available = s.parse().unwrap_or(0);
                }
            }
        }
    }

    if total > 0 {
        let used = total - available;
        let usage_percent = (used as f64 / total as f64) * 100.0;
        let bar = create_bar(usage_percent);

        // Convert to GB (decimal: 1 KB = 1000 bytes, meminfo reports in KB)
        let used_gb = used as f64 / 1_000_000.0;
        let total_gb = total as f64 / 1_000_000.0;

        return format!(" {} {:.0}GB/{:.0}GB", bar, used_gb, total_gb);
    }
    "unknown".to_string()
}

// Get storage usage for all physical disks using statvfs syscall.
// Reads /proc/mounts and uses statvfs for each real filesystem - much faster than spawning df
pub fn storage() -> String {
    let mut total_bytes: u64 = 0;
    let mut used_bytes: u64 = 0;
    let mut seen_devices = std::collections::HashSet::new();

    // Read /proc/mounts as bytes for SIMD-accelerated parsing
    if let Ok(content) = fs::read("/proc/mounts") {
        let mut start = 0;
        for end in memchr_iter(b'\n', &content) {
            let line = &content[start..end];
            start = end + 1;

            // Find first space (device ends here)
            let Some(space1) = memchr::memchr(b' ', line) else {
                continue;
            };
            let device = &line[..space1];

            // Find second space (mount point ends here)
            let rest = &line[space1 + 1..];
            let Some(space2) = memchr::memchr(b' ', rest) else {
                continue;
            };
            let mount_point_bytes = &rest[..space2];

            // Filter for real disks: starts with /dev/ and not loop devices
            if device.len() < 5
                || &device[..5] != b"/dev/"
                || memmem::find(device, b"/loop").is_some()
            {
                continue;
            }

            let Ok(device_str) = std::str::from_utf8(device) else {
                continue;
            };
            let Ok(mount_point) = std::str::from_utf8(mount_point_bytes) else {
                continue;
            };

            // Avoid double counting if device mounted multiple times
            if !seen_devices.insert(device_str.to_string()) {
                continue;
            }

            // Use statvfs syscall to get filesystem stats
            if let Some((total, used)) = get_fs_stats(mount_point) {
                total_bytes += total;
                used_bytes += used;
            }
        }
    }

    if total_bytes > 0 {
        let usage_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;
        let bar = create_bar(usage_percent);

        // Convert to GB (decimal: 1 GB = 1,000,000,000 bytes)
        let used_gb = used_bytes as f64 / 1_000_000_000.0;
        let total_gb = total_bytes as f64 / 1_000_000_000.0;

        // Use TB for total if >= 1000GB, frees up horizontal line space
        if total_gb >= 1000.0 {
            let total_tb = total_gb / 1000.0;
            // Trim .00 if it's a whole number (e.g., 1.00TB -> 1TB)
            let total_str = if (total_tb - total_tb.round()).abs() < 0.005 {
                format!("{}TB", total_tb.round() as u64)
            } else {
                format!("{:.2}TB", total_tb)
            };
            return format!("{} {:.0}GB/{}", bar, used_gb, total_str);
        }

        return format!("{} {:.0}GB/{:.0}GB", bar, used_gb, total_gb);
    }
    "unknown".to_string()
}

// Get filesystem stats using statvfs syscall
// Returns (total_bytes, used_bytes) or None on failure
fn get_fs_stats(path: &str) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new(path).ok()?;
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();

    // SAFETY: statvfs is a standard POSIX syscall, c_path is valid null-terminated string
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };

    if result != 0 {
        return None;
    }

    // SAFETY: statvfs succeeded, stat is now initialized
    let stat = unsafe { stat.assume_init() };

    let block_size = stat.f_frsize as u64;
    let total_blocks = stat.f_blocks as u64;
    let free_blocks = stat.f_bfree as u64;

    let total = total_blocks * block_size;
    let used = (total_blocks - free_blocks) * block_size;

    Some((total, used))
}

// Get battery status if device is a laptop (chassis check)
pub fn laptop_battery() -> String {
    if !is_laptop() {
        return "unknown".to_string();
    }

    // Find first available battery (usually BAT0 or BAT1)
    let power_supply = std::path::Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("BAT") {
                let path = entry.path();

                // Get capacity
                let capacity = read_first_line(path.join("capacity").to_str().unwrap_or(""))
                    .and_then(|c| c.parse::<u8>().ok())
                    .unwrap_or(0);

                // Get status
                let status = read_first_line(path.join("status").to_str().unwrap_or(""))
                    .unwrap_or_else(|| "Unknown".to_string());

                let nerd = get_cached_is_nerd_font();
                let status_icon: &str = match status.as_str() {
                    "Charging" => if nerd { "󰂐" } else { "(+)" },
                    "Discharging" => if nerd { "󰂍" } else { "(-)" },
                    _ => &status,
                };

                let bar = create_bar(capacity as f64);

                return format!("{} {}% {}", bar, capacity, status_icon);
            }
        }
    }

    "unknown".to_string()
}

// Get screen resolution and refresh rate
// Returns a Vec of (key, value) pairs for each monitor, primary first
// Tries DRM ioctl first (fastest), then xrandr, then Wayland-specific methods
pub fn screen() -> Vec<(String, String)> {
    // Try DRM ioctl first - fastest method (~200µs vs ~5ms for subprocess)
    if let Some(screens) = screen_from_drm() {
        return screens;
    }

    // Try xrandr (works on X11 and XWayland)
    if let Some(screens) = screen_from_xrandr() {
        return screens;
    }

    // Wayland fallbacks based on compositor/DE
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();

    // Niri compositor
    if desktop == "niri" || std::env::var("NIRI_SOCKET").is_ok() {
        if let Some(screens) = screen_from_niri() {
            return screens;
        }
    }

    // GNOME via D-Bus
    if desktop.contains("gnome") {
        if let Some(screens) = screen_from_gnome_dbus() {
            return screens;
        }
    }

    vec![]
}

// DRM ioctl structures for getting connector mode info and rotation
#[repr(C)]
struct DrmModeGetConnector {
    encoders_ptr: u64,
    modes_ptr: u64,
    props_ptr: u64,
    prop_values_ptr: u64,
    count_modes: u32,
    count_props: u32,
    count_encoders: u32,
    encoder_id: u32,
    connector_id: u32,
    connector_type: u32,
    connector_type_id: u32,
    connection: u32,
    mm_width: u32,
    mm_height: u32,
    subpixel: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct DrmModeModeinfo {
    clock: u32,
    hdisplay: u16,
    hsync_start: u16,
    hsync_end: u16,
    htotal: u16,
    hskew: u16,
    vdisplay: u16,
    vsync_start: u16,
    vsync_end: u16,
    vtotal: u16,
    vscan: u16,
    vrefresh: u32,
    flags: u32,
    type_: u32,
    name: [u8; 32],
}

#[repr(C)]
struct DrmModeGetEncoder {
    encoder_id: u32,
    encoder_type: u32,
    crtc_id: u32,
    possible_crtcs: u32,
    possible_clones: u32,
}

#[repr(C)]
struct DrmModeGetCrtc {
    set_connectors_ptr: u64,
    count_connectors: u32,
    crtc_id: u32,
    fb_id: u32,
    x: u32,
    y: u32,
    gamma_size: u32,
    mode_valid: u32,
    mode: DrmModeModeinfo,
}

#[repr(C)]
struct DrmModeGetPlaneRes {
    plane_id_ptr: u64,
    count_planes: u32,
}

#[repr(C)]
struct DrmModeGetPlane {
    plane_id: u32,
    crtc_id: u32,
    fb_id: u32,
    possible_crtcs: u32,
    gamma_size: u32,
    count_format_types: u32,
    format_type_ptr: u64,
}

#[repr(C)]
struct DrmModeObjGetProperties {
    props_ptr: u64,
    prop_values_ptr: u64,
    count_props: u32,
    obj_id: u32,
    obj_type: u32,
}

#[repr(C)]
struct DrmModeGetProperty {
    values_ptr: u64,
    enum_blob_ptr: u64,
    prop_id: u32,
    flags: u32,
    name: [u8; 32],
    count_values: u32,
    count_enum_blobs: u32,
}

// DRM ioctl numbers
const DRM_IOCTL_MODE_GETCONNECTOR: libc::c_ulong = 0xc05064a7;
const DRM_IOCTL_MODE_GETENCODER: libc::c_ulong = 0xc01464a6;
const DRM_IOCTL_MODE_GETCRTC: libc::c_ulong = 0xc06864a1;
const DRM_IOCTL_MODE_GETPLANERESOURCES: libc::c_ulong = 0xc01064b5;
const DRM_IOCTL_MODE_GETPLANE: libc::c_ulong = 0xc02064b6;
const DRM_IOCTL_MODE_OBJ_GETPROPERTIES: libc::c_ulong = 0xc02064b9;
const DRM_IOCTL_MODE_GETPROPERTY: libc::c_ulong = 0xc04064aa;
const DRM_IOCTL_SET_CLIENT_CAP: libc::c_ulong = 0x4010640d;

// DRM object types
const DRM_MODE_OBJECT_PLANE: u32 = 0xeeeeeeee;

// DRM rotation values (bitmask)
const DRM_MODE_ROTATE_0: u64 = 1 << 0;
const DRM_MODE_ROTATE_90: u64 = 1 << 1;
const DRM_MODE_ROTATE_270: u64 = 1 << 3;

// Get rotation for a CRTC by checking its primary plane's rotation property
fn get_crtc_rotation(fd: i32, crtc_id: u32) -> u64 {
    // Enable universal planes to access all planes including primary
    let cap: [u64; 2] = [2, 1]; // DRM_CLIENT_CAP_UNIVERSAL_PLANES = 2, value = 1
    unsafe { libc::ioctl(fd, DRM_IOCTL_SET_CLIENT_CAP, cap.as_ptr()) };

    // Get plane resources
    let mut plane_res = DrmModeGetPlaneRes {
        plane_id_ptr: 0,
        count_planes: 0,
    };

    if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_GETPLANERESOURCES, &mut plane_res) } < 0 {
        return DRM_MODE_ROTATE_0;
    }

    if plane_res.count_planes == 0 {
        return DRM_MODE_ROTATE_0;
    }

    let mut plane_ids: Vec<u32> = vec![0; plane_res.count_planes as usize];
    plane_res.plane_id_ptr = plane_ids.as_mut_ptr() as u64;

    if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_GETPLANERESOURCES, &mut plane_res) } < 0 {
        return DRM_MODE_ROTATE_0;
    }

    // Find the plane associated with this CRTC
    for &plane_id in &plane_ids {
        let mut plane = DrmModeGetPlane {
            plane_id,
            crtc_id: 0,
            fb_id: 0,
            possible_crtcs: 0,
            gamma_size: 0,
            count_format_types: 0,
            format_type_ptr: 0,
        };

        if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_GETPLANE, &mut plane) } < 0 {
            continue;
        }

        // Check if this plane is attached to our CRTC
        if plane.crtc_id != crtc_id {
            continue;
        }

        // Get plane properties to find rotation
        let mut obj_props = DrmModeObjGetProperties {
            props_ptr: 0,
            prop_values_ptr: 0,
            count_props: 0,
            obj_id: plane_id,
            obj_type: DRM_MODE_OBJECT_PLANE,
        };

        if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_OBJ_GETPROPERTIES, &mut obj_props) } < 0 {
            continue;
        }

        if obj_props.count_props == 0 {
            continue;
        }

        let mut props: Vec<u32> = vec![0; obj_props.count_props as usize];
        let mut prop_values: Vec<u64> = vec![0; obj_props.count_props as usize];
        obj_props.props_ptr = props.as_mut_ptr() as u64;
        obj_props.prop_values_ptr = prop_values.as_mut_ptr() as u64;

        if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_OBJ_GETPROPERTIES, &mut obj_props) } < 0 {
            continue;
        }

        // Look for "rotation" property
        for i in 0..obj_props.count_props as usize {
            let mut prop = DrmModeGetProperty {
                values_ptr: 0,
                enum_blob_ptr: 0,
                prop_id: props[i],
                flags: 0,
                name: [0; 32],
                count_values: 0,
                count_enum_blobs: 0,
            };

            if unsafe { libc::ioctl(fd, DRM_IOCTL_MODE_GETPROPERTY, &mut prop) } < 0 {
                continue;
            }

            let name = std::str::from_utf8(&prop.name)
                .unwrap_or("")
                .trim_end_matches('\0');

            if name == "rotation" {
                return prop_values[i];
            }
        }
    }

    DRM_MODE_ROTATE_0
}

// Get screen info using DRM ioctl - much faster than spawning subprocesses
fn screen_from_drm() -> Option<Vec<(String, String)>> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    // Try opening DRM cards in order (card0, card1, card2) to find the active GPU
    let (fd, card_num) = (0..3)
        .find_map(|i| {
            File::open(format!("/dev/dri/card{}", i))
                .ok()
                .map(|f| (f, i))
        })?;
    let raw_fd = fd.as_raw_fd();

    // Create prefix for filtering sysfs entries (e.g., "card1-")
    let card_prefix = format!("card{}-", card_num);

    let mut screens: Vec<(bool, String)> = Vec::new();

    // Find connected displays via sysfs, get modes via ioctl
    for entry in fs::read_dir("/sys/class/drm").ok()?.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Only process connector entries like card1-eDP-1, card1-DP-1
        if !name_str.starts_with(&card_prefix) || name_str.contains("Writeback") {
            continue;
        }

        let path = entry.path();
        let status = fs::read_to_string(path.join("status")).ok()?;

        if status.trim() != "connected" {
            continue;
        }

        // Get connector_id from sysfs
        let connector_id: u32 = fs::read_to_string(path.join("connector_id"))
            .ok()?
            .trim()
            .parse()
            .ok()?;

        // Get connector info via DRM ioctl
        let mut conn = DrmModeGetConnector {
            encoders_ptr: 0,
            modes_ptr: 0,
            props_ptr: 0,
            prop_values_ptr: 0,
            count_modes: 0,
            count_props: 0,
            count_encoders: 0,
            encoder_id: 0,
            connector_id,
            connector_type: 0,
            connector_type_id: 0,
            connection: 0,
            mm_width: 0,
            mm_height: 0,
            subpixel: 0,
            _pad: 0,
        };

        // First ioctl call to get counts
        if unsafe { libc::ioctl(raw_fd, DRM_IOCTL_MODE_GETCONNECTOR, &mut conn) } < 0 {
            continue;
        }

        if conn.count_modes == 0 {
            continue;
        }

        // Allocate buffers for all arrays
        let mut modes: Vec<DrmModeModeinfo> = vec![DrmModeModeinfo::default(); conn.count_modes as usize];
        let mut encoders: Vec<u32> = vec![0u32; conn.count_encoders as usize];
        let mut props: Vec<u32> = vec![0u32; conn.count_props as usize];
        let mut prop_values: Vec<u64> = vec![0u64; conn.count_props as usize];

        conn.modes_ptr = modes.as_mut_ptr() as u64;
        conn.encoders_ptr = encoders.as_mut_ptr() as u64;
        conn.props_ptr = props.as_mut_ptr() as u64;
        conn.prop_values_ptr = prop_values.as_mut_ptr() as u64;

        // Second ioctl call to get actual data
        if unsafe { libc::ioctl(raw_fd, DRM_IOCTL_MODE_GETCONNECTOR, &mut conn) } < 0 {
            continue;
        }

        // Get CRTC ID via encoder to get current mode and rotation
        let mut crtc_id: u32 = 0;
        if conn.encoder_id != 0 {
            let mut encoder = DrmModeGetEncoder {
                encoder_id: conn.encoder_id,
                encoder_type: 0,
                crtc_id: 0,
                possible_crtcs: 0,
                possible_clones: 0,
            };
            if unsafe { libc::ioctl(raw_fd, DRM_IOCTL_MODE_GETENCODER, &mut encoder) } >= 0 {
                crtc_id = encoder.crtc_id;
            }
        }

        // Skip if no CRTC (display not active)
        if crtc_id == 0 {
            continue;
        }

        // Get the current mode from CRTC (the actually active mode, not just preferred)
        let mut crtc = DrmModeGetCrtc {
            set_connectors_ptr: 0,
            count_connectors: 0,
            crtc_id,
            fb_id: 0,
            x: 0,
            y: 0,
            gamma_size: 0,
            mode_valid: 0,
            mode: DrmModeModeinfo::default(),
        };

        if unsafe { libc::ioctl(raw_fd, DRM_IOCTL_MODE_GETCRTC, &mut crtc) } < 0 {
            continue;
        }

        // If CRTC has no valid mode, skip (shouldn't happen for connected displays)
        if crtc.mode_valid == 0 {
            continue;
        }

        let mode = &crtc.mode;

        // Check rotation from CRTC's plane
        let rotation = get_crtc_rotation(raw_fd, crtc_id);

        let is_primary = name_str.contains("eDP");

        // Determine portrait mode: 90° or 270° rotation, or physical portrait panel
        let is_portrait = (rotation & (DRM_MODE_ROTATE_90 | DRM_MODE_ROTATE_270)) != 0
            || mode.vdisplay > mode.hdisplay;

        // Calculate refresh rate from timing parameters if vrefresh is 0 or missing
        // Formula: refresh = (clock * 1000) / (htotal * vtotal)
        // clock is in kHz, so multiply by 1000 to get Hz
        let refresh = if mode.vrefresh > 0 {
            mode.vrefresh
        } else if mode.htotal > 0 && mode.vtotal > 0 {
            let htotal = mode.htotal as u64;
            let vtotal = mode.vtotal as u64;
            let clock = mode.clock as u64;
            // clock is in kHz, multiply by 1000 to get Hz, then divide by total pixels
            ((clock * 1000) / (htotal * vtotal)) as u32
        } else {
            0
        };

        let icon = if is_portrait { if get_cached_is_nerd_font() { "󰆡" } else { "Portrait" } } else { if get_cached_is_nerd_font() { "󰏠" } else { "Landscape" } };
        let display_str = format!(
            "{} {}x{} @ {}Hz",
            icon, mode.hdisplay, mode.vdisplay, refresh
        );
        screens.push((is_primary, display_str));
    }

    if screens.is_empty() {
        return None;
    }

    Some(format_screens(screens))
}

// Parse xrandr --current output
fn screen_from_xrandr() -> Option<Vec<(String, String)>> {
    let output = Command::new("xrandr")
        .arg("--current")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = &output.stdout;
    let mut screens: Vec<(bool, String)> = Vec::new();
    let mut current_is_primary = false;
    let mut current_is_portrait = false;

    let mut start = 0;
    for end in memchr_iter(b'\n', stdout) {
        let line = &stdout[start..end];
        start = end + 1;

        if memmem::find(line, b" connected").is_some() {
            current_is_primary = memmem::find(line, b" primary ").is_some();
            let before_paren = memchr::memchr(b'(', line)
                .map(|p| &line[..p])
                .unwrap_or(line);
            current_is_portrait = memmem::find(before_paren, b" left").is_some()
                || memmem::find(before_paren, b" right").is_some();
        } else if memchr::memchr(b'*', line).is_some() {
            let line_str = std::str::from_utf8(line).ok()?;
            let parts: Vec<&str> = line_str.split_whitespace().collect();
            if parts.len() >= 2 {
                let res = parts[0];
                let rate: String = parts[1]
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.')
                    .collect();

                let icon = if current_is_portrait { if get_cached_is_nerd_font() { "󰆡" } else { "Portrait" } } else { if get_cached_is_nerd_font() { "󰏠" } else { "Landscape" } };
                let display_str = if let Ok(rate_f) = rate.parse::<f64>() {
                    format!("{} {} @ {}Hz", icon, res, rate_f.round() as u64)
                } else {
                    format!("{} {} @ {}Hz", icon, res, rate)
                };
                screens.push((current_is_primary, display_str));
            }
        }
    }

    if screens.is_empty() {
        return None;
    }

    Some(format_screens(screens))
}

// Parse niri msg outputs
fn screen_from_niri() -> Option<Vec<(String, String)>> {
    let output = Command::new("niri")
        .args(["msg", "outputs"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = &output.stdout;
    let mut screens: Vec<(bool, String)> = Vec::new();
    let mut current_is_portrait = false;
    let mut is_first = true;

    let mut start = 0;
    for end in memchr_iter(b'\n', stdout) {
        let line = &stdout[start..end];
        start = end + 1;

        // Output line starts with "Output " (no leading whitespace)
        if line.starts_with(b"Output ") {
            // First output is treated as primary
            is_first = screens.is_empty();
            current_is_portrait = false;
        }
        // Transform line: "  Transform: 90° counter-clockwise" or "  Transform: normal"
        else if memmem::find(line, b"Transform:").is_some() {
            // Portrait if rotated 90° or 270°
            current_is_portrait = memmem::find(line, b"90").is_some()
                || memmem::find(line, b"270").is_some();
        }
        // Current mode line: "  Current mode: 2560x1440 @ 74.968 Hz"
        else if memmem::find(line, b"Current mode:").is_some() {
            let line_str = std::str::from_utf8(line).ok()?;
            // Extract resolution and refresh rate
            if let Some(mode_start) = line_str.find("Current mode:") {
                let mode_part = &line_str[mode_start + 13..].trim();
                let parts: Vec<&str> = mode_part.split_whitespace().collect();
                // Expected: ["2560x1440", "@", "74.968", "Hz"]
                if parts.len() >= 3 {
                    let res = parts[0];
                    let rate = parts[2];

                    let icon = if current_is_portrait { if get_cached_is_nerd_font() { "󰆡" } else { "Portrait" } } else { if get_cached_is_nerd_font() { "󰏠" } else { "Landscape" } };
                    let display_str = if let Ok(rate_f) = rate.parse::<f64>() {
                        format!("{} {} @ {}Hz", icon, res, rate_f.round() as u64)
                    } else {
                        format!("{} {} @ {}Hz", icon, res, rate)
                    };
                    screens.push((is_first, display_str));
                }
            }
        }
    }

    if screens.is_empty() {
        return None;
    }

    Some(format_screens(screens))
}

// Get screen info from GNOME via D-Bus
fn screen_from_gnome_dbus() -> Option<Vec<(String, String)>> {
    // Query Mutter's DisplayConfig D-Bus interface for monitor information
    // This works on native GNOME Wayland sessions
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest=org.gnome.Mutter.DisplayConfig",
            "--object-path=/org/gnome/Mutter/DisplayConfig",
            "--method=org.gnome.Mutter.DisplayConfig.GetCurrentState",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut screens: Vec<(bool, String)> = Vec::new();

    // The D-Bus output format for each mode is a tuple:
    // ('2880x1800@120.000', 2880, 1800, 119.999, 2.0, [...], {'is-current': <true>, ...})
    // We look for modes that have 'is-current': <true> in their properties dict

    // Find all occurrences of 'is-current': <true> and extract the mode info before it
    let mut search_start = 0;
    while let Some(current_pos) = stdout[search_start..].find("'is-current': <true>") {
        let abs_pos = search_start + current_pos;

        // Look backwards to find the mode tuple - find the mode string pattern
        // The mode string looks like ('2880x1800@120.000', ...)
        let region_start = abs_pos.saturating_sub(300);
        let region = &stdout[region_start..abs_pos];

        // Find the last mode pattern like '2880x1800@120.000' before this is-current
        // Mode format: 'WIDTHxHEIGHT@RATE'
        if let Some(mode_match) = region.rfind("('") {
            let mode_start = region_start + mode_match + 2;
            if let Some(mode_end) = stdout[mode_start..].find("',") {
                let mode_str = &stdout[mode_start..mode_start + mode_end];
                // Parse 'WIDTHxHEIGHT@RATE' format
                if let Some(at_pos) = mode_str.find('@') {
                    let res = &mode_str[..at_pos];
                    let rate = &mode_str[at_pos + 1..];

                    // Check for primary in the logical monitors section
                    // The logical monitor tuple format: (x, y, scale, transform, primary, monitors, props)
                    // where primary is a boolean (true/false)
                    // The 5th element (index 4) is the primary boolean
                    let after_region = &stdout[abs_pos..(abs_pos + 500).min(stdout.len())];
                    let before_region = &stdout[region_start..abs_pos];

                    // For primary: look for ", true, [(" pattern which indicates primary=true before monitors list
                    let is_primary =
                        before_region.contains(", true, [('") || after_region.contains(", true, [('");

                    // Transform values: 0=normal, 1=90°, 2=180°, 3=270°
                    // Portrait if transform is 1 or 3
                    // The logical monitors section has format: (x, y, scale, uint32 TRANSFORM, primary, ...)
                    // We need to check for " uint32 1," or " uint32 3," with a space before to avoid matching 0, 10, etc.
                    let is_portrait = before_region.contains(" uint32 1,")
                        || before_region.contains(" uint32 3,")
                        || after_region.contains(" uint32 1,")
                        || after_region.contains(" uint32 3,");

                    if let Ok(rate_f) = rate.parse::<f64>() {
                        let icon = if is_portrait { if get_cached_is_nerd_font() { "󰆡" } else { "Portrait" } } else { if get_cached_is_nerd_font() { "󰏠" } else { "Landscape" } };
                        let display_str =
                            format!("{} {} @ {}Hz", icon, res, rate_f.round() as u64);
                        screens.push((is_primary, display_str));
                    }
                }
            }
        }

        search_start = abs_pos + 20;
    }

    if screens.is_empty() {
        return None;
    }

    Some(format_screens(screens))
}

// Format screens list into the output format (primary first, tree-style for multiple)
fn format_screens(mut screens: Vec<(bool, String)>) -> Vec<(String, String)> {
    // Sort so primary monitor comes first
    screens.sort_by(|a, b| b.0.cmp(&a.0));

    if screens.len() == 1 {
        return vec![("Display".to_string(), screens[0].1.clone())];
    }

    // Multiple monitors: header line + tree-style entries
    let mut result = vec![("Displays".to_string(), String::new())];
    let last_idx = screens.len() - 1;
    for (i, (_, s)) in screens.iter().enumerate() {
        if i == last_idx {
            result.push(("╰─".to_string(), s.clone()));
        } else {
            result.push(("├─".to_string(), s.clone()));
        }
    }
    result
}
