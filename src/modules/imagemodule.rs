// Image handling module for Slowfetch
// Supports Kitty graphics protocol for Ghostty, Kitty, and Konsole terminals

use std::env;
use std::path::Path;

// Default slowfetch image embedded in the binary
const DEFAULT_IMAGE: &[u8] = include_bytes!("../assets/default/slowfetch.png");

// Check if running in Konsole (requires direct transmission)
fn is_konsole() -> bool {
    env::var("KONSOLE_VERSION").is_ok()
}

// Display an image using the Kitty graphics protocol.
// Automatically handles animated GIFs vs static images.
pub fn display_image(path: &Path, box_cols: u16, box_rows: u16) -> Result<String, String> {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current dir: {e}"))?
            .join(path)
    };
//decoding, which is much faster than transmitting raw pixel data. The temp file write is a one-time cost and the file gets reused/overwritten on subsequent runs.

    if !abs_path.exists() {
        return Err(format!("Image file not found: {}", abs_path.display()));
    }

    if is_gif(&abs_path) {
        display_kitty_gif(&abs_path, box_cols, box_rows)
    } else {
        display_kitty_static(&abs_path, box_cols, box_rows)
    }
}

// Display a static image using Kitty protocol
fn display_kitty_static(path: &Path, box_cols: u16, box_rows: u16) -> Result<String, String> {
    // Konsole only supports direct transmission, not file-based
    if is_konsole() {
        return display_kitty_direct(path, box_cols, box_rows);
    }

    // Use file-based transmission for terminals that support it (faster)
    let action = kitty_image::Action::TransmitAndDisplay(
        kitty_image::ActionTransmission {
            format: kitty_image::Format::Png,
            medium: kitty_image::Medium::File,
            ..Default::default()
        },
        kitty_image::ActionPut {
            columns: box_cols as u32,
            rows: box_rows as u32,
            ..Default::default()
        },
    );

    let command = kitty_image::Command::with_payload_from_path(action, path);
    Ok(kitty_image::WrappedCommand::new(command).to_string())
}

// Display image using direct transmission for terminals like Konsole
fn display_kitty_direct(path: &Path, box_cols: u16, box_rows: u16) -> Result<String, String> {
    use std::io::Write;

    // Load and encode image as PNG
    let img = image::open(path).map_err(|e| format!("Failed to load image: {e}"))?;
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    let action = kitty_image::Action::TransmitAndDisplay(
        kitty_image::ActionTransmission {
            format: kitty_image::Format::Rgba32,
            medium: kitty_image::Medium::Direct,
            width,
            height,
            ..Default::default()
        },
        kitty_image::ActionPut {
            columns: box_cols as u32,
            rows: box_rows as u32,
            ..Default::default()
        },
    );

    let mut command = kitty_image::Command::new(action);
    command.payload = rgba.into_raw().into();

    // Use chunked sending for large payloads
    let wrapped = kitty_image::WrappedCommand::new(command);
    let mut stdout = std::io::stdout().lock();
    wrapped
        .send_chunked(&mut stdout)
        .map_err(|e| format!("Failed to send image: {e}"))?;
    let _ = stdout.flush();

    Ok(String::new())
}

// Display an animated GIF using kitten icat
fn display_kitty_gif(path: &Path, box_cols: u16, box_rows: u16) -> Result<String, String> {
    use std::process::{Command, Stdio};

    let (col, row) = get_cursor_position().unwrap_or((1, 1));

    let status = Command::new("kitten")
        .args([
            "icat",
            "--stdin=no",
            "--scale-up",
            &format!("--place={}x{}@{}x{}", box_cols, box_rows, col - 1, row - 1),
            "--loop=-1",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run kitten icat: {e}"))?;

    if status.success() {
        Ok(String::new())
    } else {
        Err("kitten icat failed".into())
    }
}

// Display the embedded default slowfetch image
pub fn display_default_image(box_cols: u16, box_rows: u16) -> Result<String, String> {
    // For Konsole, use direct transmission (it doesn't support file-based)
    if is_konsole() {
        return display_image_bytes(DEFAULT_IMAGE, box_cols, box_rows);
    }

    // For Kitty/Ghostty, write to cache file and use fast file-based transmission
    let cache_dir = env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            env::var("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".cache"))
                .unwrap_or_else(|_| std::env::temp_dir())
        })
        .join("slowfetch");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {e}"))?;
    let cache_path = cache_dir.join("default.png");
    std::fs::write(&cache_path, DEFAULT_IMAGE)
        .map_err(|e| format!("Failed to write cache image: {e}"))?;

    let action = kitty_image::Action::TransmitAndDisplay(
        kitty_image::ActionTransmission {
            format: kitty_image::Format::Png,
            medium: kitty_image::Medium::File,
            ..Default::default()
        },
        kitty_image::ActionPut {
            columns: box_cols as u32,
            rows: box_rows as u32,
            ..Default::default()
        },
    );

    let command = kitty_image::Command::with_payload_from_path(action, &cache_path);
    Ok(kitty_image::WrappedCommand::new(command).to_string())
}

// Display an image from raw bytes using Kitty protocol
fn display_image_bytes(data: &[u8], box_cols: u16, box_rows: u16) -> Result<String, String> {
    use std::io::Write;

    let img = image::load_from_memory(data).map_err(|e| format!("Failed to load image: {e}"))?;
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    let action = kitty_image::Action::TransmitAndDisplay(
        kitty_image::ActionTransmission {
            format: kitty_image::Format::Rgba32,
            medium: kitty_image::Medium::Direct,
            width,
            height,
            ..Default::default()
        },
        kitty_image::ActionPut {
            columns: box_cols as u32,
            rows: box_rows as u32,
            ..Default::default()
        },
    );

    let mut command = kitty_image::Command::new(action);
    command.payload = rgba.into_raw().into();

    let wrapped = kitty_image::WrappedCommand::new(command);
    let mut stdout = std::io::stdout().lock();
    wrapped
        .send_chunked(&mut stdout)
        .map_err(|e| format!("Failed to send image: {e}"))?;
    let _ = stdout.flush();

    Ok(String::new())
}

fn is_gif(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("gif"))
}

// Query current cursor position using ANSI DSR
fn get_cursor_position() -> Option<(u16, u16)> {
    use std::io::{Read, Write};

    let mut termios: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(0, &mut termios) } != 0 {
        return None;
    }
    let original = termios;

    termios.c_lflag &= !(libc::ICANON | libc::ECHO);
    termios.c_cc[libc::VMIN] = 0;
    termios.c_cc[libc::VTIME] = 1;

    if unsafe { libc::tcsetattr(0, libc::TCSANOW, &termios) } != 0 {
        return None;
    }

    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"\x1b[6n");
    let _ = stdout.flush();

    let mut buf = [0u8; 32];
    let mut len = 0;
    for byte in std::io::stdin().lock().bytes().flatten() {
        buf[len] = byte;
        len += 1;
        if byte == b'R' || len >= 31 {
            break;
        }
    }

    unsafe { libc::tcsetattr(0, libc::TCSANOW, &original) };

    let s = std::str::from_utf8(&buf[..len]).ok()?;
    let coords = s.strip_prefix("\x1b[")?.strip_suffix('R')?;
    let (row, col) = coords.split_once(';')?;
    Some((col.parse().ok()?, row.parse().ok()?))
}
