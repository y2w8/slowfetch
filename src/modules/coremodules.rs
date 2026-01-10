// Core system information modules for Slowfetch.

use std::fs;

use crate::cache;
use crate::helpers::read_first_line;

// Get the OS name from /etc/os-release.
// Uses persistent cache to avoid repeated file reads.
pub fn os() -> String {
    // Check cache first (unless --refresh was passed)
    if let Some(cached) = cache::get_cached_os() {
        return cached;
    }

    // No cache hit, fetch fresh value
    let result = os_fresh();

    // Cache the result for next time
    cache::cache_os(&result);

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
        // Fast path: read directly from /proc/1/comm (no subprocess spawn)
        if let Ok(comm) = fs::read_to_string("/proc/1/comm") {
            return comm.trim().to_string();
        }
    }

    // Fallback for macOS or if /proc isn't available
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(["-p", "1", "-o", "comm="])
            .output()
        {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }

    "unknown".to_string()
}
