// Core system information modules for Slowfetch.

use std::fs;

use crate::cache;
use crate::helpers::read_first_line;

// Check if the system is an immutable OS by examining /etc/os-release
fn is_immutable_os() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let pretty_name = line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches(|c| c == '"' || c == '\'');
                    return pretty_name.contains("Silverblue");
                }
            }
        }
    }
    false
}

// Get the OS name from /etc/os-release.
// Uses persistent cache to avoid repeated file reads.
// Note: Caching is disabled for immutable OSes since the OS name changes daily.
pub fn os() -> String {
    // Check if its an immutable OS first
    let is_immutable = is_immutable_os();

    // Check cache first (unless --refresh was passed or on immutable OS)
    if !is_immutable {
        if let Some(cached) = cache::get_cached_os() {
            return cached;
        }
    }

    // No cache hit, fetch fresh value
    let result = os_fresh();

    // Cache the result for next time (unless on immutable OS)
    if !is_immutable {
        cache::cache_os(&result);
    }

    result
}

// Fetch OS info fresh (no cache)
fn os_fresh() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sw_vers")
            .arg("-productName")
            .output()
        {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(version_output) = Command::new("sw_vers")
                .arg("-productVersion")
                .output()
            {
                let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
                if !name.is_empty() && !version.is_empty() {
                    return format!("{} {}", name, version);
                }
            }
        }
        return "macOS".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string();
                }
            }
        }
        return "Linux".to_string();
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        "unsupported platform".to_string()
    }
}

// Get the kernel version
pub fn kernel() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("uname").arg("-r").output() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return version;
            }
        }
        return "unknown".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        read_first_line("/proc/sys/kernel/osrelease").unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        "unsupported platform".to_string()
    }
}

// Get the system uptime
pub fn uptime() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("uptime").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // macOS uptime format: " 10:30  up 2 days,  3:45, 2 users, ..."
            // or: " 10:30  up  3:45, 2 users, ..." (less than a day)
            if let Some(up_idx) = output_str.find("up ") {
                let after_up = &output_str[up_idx + 3..];
                if let Some(users_idx) = after_up.find(" user") {
                    let uptime_part = after_up[..users_idx].trim();
                    // Remove trailing comma and user count
                    let uptime_part = uptime_part.trim_end_matches(|c: char| c.is_ascii_digit() || c == ',' || c == ' ');
                    let uptime_part = uptime_part.trim_end_matches(',').trim();
                    if !uptime_part.is_empty() {
                        return uptime_part.to_string();
                    }
                }
            }
        }
        return "unknown".to_string();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/uptime") {
            if let Some(seconds_str) = content.split_whitespace().next() {
                if let Ok(seconds) = seconds_str.parse::<f64>() {
                    let s = seconds as u64;
                    let h = s / 3600;
                    let m = (s % 3600) / 60;
                    if h > 0 {
                        return format!("{}h {}m", h, m);
                    } else {
                        return format!("{}m", m);
                    }
                }
            }
        }
        return "unknown".to_string();
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        "unsupported platform".to_string()
    }
}

// Get the init system (PID 1 process name)
// Uses persistent cache to avoid repeated detection (init system doesn't change)
pub fn init() -> String {
    // Check cache first (unless --refresh was passed)
    if let Some(cached) = cache::get_cached_init() {
        return cached;
    }

    // No cache hit, fetch fresh value
    let result = init_fresh();

    // Cache the result for next time
    cache::cache_init(&result);

    result
}

// Fetch init system info fresh (no cache)
fn init_fresh() -> String {
    #[cfg(target_os = "linux")]
    {
        use std::path::Path;

        // Check for non-systemd init systems by looking for their config files in /etc/
        // These checks are more reliable than /proc/1/comm which may show "init" generically

        // OpenRC (used by Gentoo, Alpine, Artix, etc.)
        if Path::new("/etc/openrc").exists() || Path::new("/etc/rc.conf").exists() {
            // Confirm it's actually OpenRC by checking for the runlevels dir
            if Path::new("/etc/runlevels").exists() {
                return "openrc".to_string();
            }
        }

        // runit (used by Void Linux, Artix, etc.)
        if Path::new("/etc/runit").exists() {
            return "runit".to_string();
        }

        // s6 (used by Artix, etc.)
        if Path::new("/etc/s6").exists() || Path::new("/etc/s6-rc").exists() {
            return "s6".to_string();
        }

        // dinit (used by Chimera Linux, Artix, etc.)
        if Path::new("/etc/dinit.d").exists() {
            return "dinit".to_string();
        }

        // SysVinit (traditional init)
        if Path::new("/etc/inittab").exists() && !Path::new("/run/systemd/system").exists() {
            return "sysvinit".to_string();
        }

        // Check if systemd is actually running (not just installed)
        if Path::new("/run/systemd/system").exists() {
            return "systemd".to_string();
        }

        // Fallback: read directly from /proc/1/comm
        if let Ok(comm) = fs::read_to_string("/proc/1/comm") {
            return comm.trim().to_string();
        }

        // Unknown non-systemd init
        "init bruv".to_string()
    }

    #[cfg(not(target_os = "linux"))]
    {
        "init bruv".to_string()
    }
}

// Get the OS installation age by checking the root filesystem birth time.
// Uses persistent cache for the birth timestamp, then calculates age from that.
pub fn os_age() -> String {
    use std::time::SystemTime;

    // Check cache first for the birth timestamp
    if let Some(birth_time) = cache::get_cached_os_birth() {
        if let Ok(now) = SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            let age_secs = now.as_secs().saturating_sub(birth_time);
            return format_age(age_secs);
        }
    }

    // No cache hit, fetch fresh value
    let birth_time = os_birth_fresh();

    if let Some(birth) = birth_time {
        // Cache the birth timestamp for next time
        cache::cache_os_birth(birth);

        if let Ok(now) = SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            let age_secs = now.as_secs().saturating_sub(birth);
            return format_age(age_secs);
        }
    }

    "unknown".to_string()
}

// Fetch OS birth timestamp fresh (no cache)
fn os_birth_fresh() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Run stat on root filesystem to get birth time
        if let Ok(output) = Command::new("stat").arg("/").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Look for the "Birth:" line in stat output
            for line in output_str.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Birth:") {
                    let birth_str = trimmed.trim_start_matches("Birth:").trim();

                    // Skip if birth time is not available (shows as "-")
                    if birth_str == "-" || birth_str.is_empty() {
                        return None;
                    }

                    // Parse the birth timestamp (format: "2024-01-15 10:30:45.123456789 +0000")
                    // Extract just the date portion for parsing
                    if let Some(date_part) = birth_str.split_whitespace().next() {
                        return parse_date_to_unix(date_part);
                    }
                }
            }
        }
        return None;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // On macOS, use stat -f %B to get birth time as unix timestamp
        if let Ok(output) = Command::new("stat").args(["-f", "%B", "/"]).output() {
            let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return timestamp_str.parse::<u64>().ok();
        }
        return None;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

// Parse a date string (YYYY-MM-DD) to Unix timestamp
fn parse_date_to_unix(date_str: &str) -> Option<u64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    // Simple calculation for days since Unix epoch (1970-01-01)
    // This is approximate but good enough for age calculation
    let days_since_epoch = days_from_civil(year, month, day)?;
    Some((days_since_epoch as u64) * 86400)
}

// Calculate days since Unix epoch from year/month/day
// Based on Howard Hinnant's algorithm
fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    // Adjust year so March is the first month
    let is_jan_or_feb = month <= 2;
    let adjusted_year = year as i64 - is_jan_or_feb as i64;
    let adjusted_month = if is_jan_or_feb { month + 9 } else { month - 3 } as u64;

    // Split into 400-year eras for leap years
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year.rem_euclid(400) as u64;

    // Day within the year
    let day_of_year = (153 * adjusted_month + 2) / 5 + day as u64 - 1;

    // Day within the 400-year era
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    // Convert to Unix epoch (days since 1970-01-01)
    Some(era * 146097 + day_of_era as i64 - 719468)
}

// Format age in seconds to human-readable string
fn format_age(seconds: u64) -> String {
    let days = seconds / 86400;
    let years = days / 365;
    let remaining_days = days % 365;
    let months = remaining_days / 30;

    if years > 0 {
        if months > 0 {
            format!("{}y {}mo", years, months)
        } else {
            format!("{}y", years)
        }
    } else if months > 0 {
        format!("{}mo", months)
    } else if days > 0 {
        format!("{}d", days)
    } else {
        "< 1d".to_string()
    }
}
