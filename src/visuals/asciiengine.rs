// ascii module for Slowfetch
// Replaces inkline and tintify
// Provides ANSI escape sequence generation for terminal colors and ASCII art rendering

use std::fmt::{self, Display};

// Terminal color representation - either 24-bit RGB or standard ANSI 16-color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
    // 24-bit RGB truecolor (r, g, b)
    Rgb(u8, u8, u8),
    // Standard ANSI 16-color palette
    Ansi(AnsiColor),
}

impl TerminalColor {
    // Pre-compute the escape sequence for this color (with bold)
    #[inline]
    fn escape_bold(self) -> EscapeCode {
        match self {
            TerminalColor::Rgb(r, g, b) => EscapeCode::from_rgb_bold(r, g, b),
            TerminalColor::Ansi(c) => EscapeCode::from_ansi_bold(c),
        }
    }

    // Pre-compute the escape sequence for this color (no bold)
    #[inline]
    fn escape(self) -> EscapeCode {
        match self {
            TerminalColor::Rgb(r, g, b) => EscapeCode::from_rgb(r, g, b),
            TerminalColor::Ansi(c) => EscapeCode::from_ansi(c),
        }
    }
}

// Pre-computed ANSI escape sequence stored inline
// Max length: "\x1b[1;38;2;255;255;255m" = 21 bytes
#[derive(Clone, Copy)]
struct EscapeCode {
    buf: [u8; 24],
    len: u8,
}

impl EscapeCode {
    const RESET: &'static str = "\x1b[0m";

    #[inline]
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let mut buf = [0u8; 24];
        let s = format!("\x1b[38;2;{r};{g};{b}m");
        let bytes = s.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        Self { buf, len: bytes.len() as u8 }
    }

    #[inline]
    fn from_rgb_bold(r: u8, g: u8, b: u8) -> Self {
        let mut buf = [0u8; 24];
        let s = format!("\x1b[1;38;2;{r};{g};{b}m");
        let bytes = s.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        Self { buf, len: bytes.len() as u8 }
    }

    #[inline]
    fn from_ansi(c: AnsiColor) -> Self {
        let mut buf = [0u8; 24];
        let code = c.fg_code();
        let s = format!("\x1b[{code}m");
        let bytes = s.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        Self { buf, len: bytes.len() as u8 }
    }

    #[inline]
    fn from_ansi_bold(c: AnsiColor) -> Self {
        let mut buf = [0u8; 24];
        let code = c.fg_code();
        let s = format!("\x1b[1;{code}m");
        let bytes = s.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
        Self { buf, len: bytes.len() as u8 }
    }

    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: We only write valid ASCII escape sequences
        unsafe { std::str::from_utf8_unchecked(&self.buf[..self.len as usize]) }
    }
}

// Standard ANSI 16-color palette (0-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AnsiColor {
    // Get the ANSI foreground color code
    #[inline]
    const fn fg_code(self) -> u8 {
        match self {
            AnsiColor::Black => 30,
            AnsiColor::Red => 31,
            AnsiColor::Green => 32,
            AnsiColor::Yellow => 33,
            AnsiColor::Blue => 34,
            AnsiColor::Magenta => 35,
            AnsiColor::Cyan => 36,
            AnsiColor::White => 37,
            AnsiColor::BrightBlack => 90,
            AnsiColor::BrightRed => 91,
            AnsiColor::BrightGreen => 92,
            AnsiColor::BrightYellow => 93,
            AnsiColor::BrightBlue => 94,
            AnsiColor::BrightMagenta => 95,
            AnsiColor::BrightCyan => 96,
            AnsiColor::BrightWhite => 97,
        }
    }
}

// Text wrapped with ANSI color escape codes, implements Display
pub struct ColoredText<'a> {
    text: &'a str,
    escape: EscapeCode,
}

impl Display for ColoredText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.escape.as_str())?;
        f.write_str(self.text)?;
        f.write_str(EscapeCode::RESET)
    }
}

// Extension trait for applying terminal colors to strings
pub trait ApplyTerminalColor {
    // Apply 24-bit RGB truecolor to text
    fn with_rgb(&self, r: u8, g: u8, b: u8) -> ColoredText<'_>;

    // Apply ANSI 16-color palette color to text
    fn with_ansi(&self, color: AnsiColor) -> ColoredText<'_>;
}

impl ApplyTerminalColor for str {
    #[inline]
    fn with_rgb(&self, r: u8, g: u8, b: u8) -> ColoredText<'_> {
        ColoredText {
            text: self,
            escape: EscapeCode::from_rgb(r, g, b),
        }
    }

    #[inline]
    fn with_ansi(&self, color: AnsiColor) -> ColoredText<'_> {
        ColoredText {
            text: self,
            escape: EscapeCode::from_ansi(color),
        }
    }
}

// Colorizes ASCII art by replacing {0}-{9} placeholders with ANSI color codes
// Implements Iterator to yield colorized lines one at a time
pub struct AsciiArtColorizer<'a> {
    lines: std::str::Lines<'a>,
    escapes: [EscapeCode; 10],
    num_colors: usize,
}

impl<'a> AsciiArtColorizer<'a> {
    // - `art`: The ASCII art string with {0}-{9} color placeholders
    // - `colors`: Slice of terminal colors to use for placeholders (index 0-9)
    // - `bold`: Whether to apply bold styling to all colors
    #[inline]
    pub fn with_colors(art: &'a str, colors: &[TerminalColor], bold: bool) -> Self {
        // Pre-compute all escape sequences upfront
        let mut escapes = [EscapeCode { buf: [0; 24], len: 0 }; 10];
        let num_colors = colors.len().min(10);

        for (i, &color) in colors.iter().take(10).enumerate() {
            escapes[i] = if bold { color.escape_bold() } else { color.escape() };
        }

        Self {
            lines: art.lines(),
            escapes,
            num_colors,
        }
    }

    // Colorize a single line by replacing placeholders with ANSI codes
    // Uses byte-level operations for speed
    #[inline]
    fn colorize_line(&self, line: &str) -> String {
        let bytes = line.as_bytes();
        let len = bytes.len();

        // Fast path: no placeholders possible
        if len < 3 {
            let mut result = String::with_capacity(len + 4);
            result.push_str(line);
            result.push_str(EscapeCode::RESET);
            return result;
        }

        // Estimate capacity: original + escape codes overhead
        let mut result = String::with_capacity(len + 128);
        let mut i = 0;

        while i < len {
            // Use memchr for fast scanning to next '{'
            if let Some(pos) = memchr::memchr(b'{', &bytes[i..]) {
                let brace_pos = i + pos;

                // Append everything before the brace
                if brace_pos > i {
                    // SAFETY: We're working with valid UTF-8 slices from the original string
                    result.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[i..brace_pos]) });
                }

                // Check if this is a valid placeholder {0}-{9}
                if brace_pos + 2 < len {
                    let digit = bytes[brace_pos + 1];
                    let close = bytes[brace_pos + 2];

                    if digit.is_ascii_digit() && close == b'}' {
                        let idx = (digit - b'0') as usize;
                        if idx < self.num_colors {
                            result.push_str(self.escapes[idx].as_str());
                            i = brace_pos + 3;
                            continue;
                        }
                    }
                }

                // Not a valid placeholder, emit the brace
                result.push('{');
                i = brace_pos + 1;
            } else {
                // No more braces, append remainder
                result.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[i..]) });
                break;
            }
        }

        result.push_str(EscapeCode::RESET);
        result
    }
}

impl Iterator for AsciiArtColorizer<'_> {
    type Item = String;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| self.colorize_line(line))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lines.size_hint()
    }
}