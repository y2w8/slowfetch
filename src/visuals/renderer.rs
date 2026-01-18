// slowfetch rendering system

use crate::configloader::{BorderLineStyle, BoxStyle};
use crate::visuals::colorcontrol::{color_border, color_key, color_title, color_value};
use crate::visuals::terminalsize::get_terminal_size;
use memchr::memchr;
use std::sync::{OnceLock, RwLock};

// Global box style config, initialized once from config file
static BOX_STYLE: OnceLock<BoxStyle> = OnceLock::new();
static BORDER_LINE_STYLE: OnceLock<BorderLineStyle> = OnceLock::new();

// Override box styles for preview mode
static PREVIEW_BOX_STYLE: RwLock<Option<BoxStyle>> = RwLock::new(None);
static PREVIEW_BORDER_LINE_STYLE: RwLock<Option<BorderLineStyle>> = RwLock::new(None);

// Initialize box styles from config - call this once at startup
pub fn init_box_styles(box_style: BoxStyle, border_line_style: BorderLineStyle) {
    let _ = BOX_STYLE.set(box_style);
    let _ = BORDER_LINE_STYLE.set(border_line_style);
}

// Set preview box styles (for TUI config preview)
pub fn set_preview_box_styles(box_style: Option<BoxStyle>, border_line_style: Option<BorderLineStyle>) {
    if let Ok(mut preview) = PREVIEW_BOX_STYLE.write() {
        *preview = box_style;
    }
    if let Ok(mut preview) = PREVIEW_BORDER_LINE_STYLE.write() {
        *preview = border_line_style;
    }
}

// Get the current box style (preview overrides base)
fn get_box_style() -> BoxStyle {
    if let Ok(preview) = PREVIEW_BOX_STYLE.read() {
        if let Some(style) = *preview {
            return style;
        }
    }
    *BOX_STYLE.get_or_init(BoxStyle::default)
}

// Get the current border line style (preview overrides base)
fn get_border_line_style() -> BorderLineStyle {
    if let Ok(preview) = PREVIEW_BORDER_LINE_STYLE.read() {
        if let Some(style) = *preview {
            return style;
        }
    }
    *BORDER_LINE_STYLE.get_or_init(BorderLineStyle::default)
}

// Get box drawing characters based on current style configuration
fn get_box_chars() -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
    let box_style = get_box_style();
    let border_line_style = get_border_line_style();

    // Use double-line corners when double lines are selected
    let (top_left, top_right, bottom_left, bottom_right) = if border_line_style == BorderLineStyle::Double {
        box_style.corners_double()
    } else {
        box_style.corners()
    };
    let (horizontal, vertical) = border_line_style.lines();

    (top_left, top_right, bottom_left, bottom_right, horizontal, vertical)
}

// Count visible characters in a byte slice (no ANSI sequences)
// Counts ASCII bytes and UTF-8 start bytes (skips continuation bytes)
#[inline]
fn count_visible_bytes(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &b in bytes {
        if b < 0x80 {
            // ASCII byte
            count += 1;
        } else if (b & 0xC0) != 0x80 {
            // UTF-8 start byte (not a continuation byte)
            count += 1;
        }
    }
    count
}

// Calculate the visible character width of a string, ignoring ANSI escape codes.
// Uses SIMD-accelerated memchr to find escape sequences quickly.
pub fn visible_len(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut visible = 0;
    let mut pos = 0;

    // Use SIMD to find each ESC (0x1b) byte
    while let Some(esc_offset) = memchr(0x1b, &bytes[pos..]) {
        let esc_pos = pos + esc_offset;
        // Count visible chars before this escape sequence
        visible += count_visible_bytes(&bytes[pos..esc_pos]);
        // Find the 'm' that ends the ANSI sequence
        if let Some(m_offset) = memchr(b'm', &bytes[esc_pos..]) {
            pos = esc_pos + m_offset + 1;
        } else {
            // Malformed: no 'm' found, skip rest of string
            return visible;
        }
    }
    // Count remaining visible chars after last escape sequence
    visible += count_visible_bytes(&bytes[pos..]);
    visible
}

// A section of system info with a title and content lines (key, value pairs).
pub struct Section {
    pub title: String,
    pub lines: Vec<(String, String)>,
}

impl Section {
    pub fn new(title: &str, lines: Vec<(String, String)>) -> Self {
        Self {
            title: title.to_string(),
            lines,
        }
    }
}

// Build a bordered box around content lines.
//
// `lines` - Content lines to display inside the box
// `title` - Optional title to display centered in the top border
// `target_width` - Optional minimum width (box expands to fit content if larger)
// `target_height` - Optional minimum height (adds vertical padding if needed)
// `center_content` - If true, center content horizontally; otherwise left-align
//
// returns a vec of strings, each representing one row of the rendered box
pub fn build_box(
    lines: &[String],
    title: Option<&str>,
    target_width: Option<usize>,
    target_height: Option<usize>,
    center_content: bool,
) -> Vec<String> {
    // --- step 1: Get box drawing characters based on current style ---
    let (box_top_left, box_top_right, box_bottom_left, box_bottom_right, box_horizontal, box_vertical) = get_box_chars();

    // --- step 2: Calculate dimensions ---

    // Pre-compute visible lengths for all lines (ignoring ANSI codes)
    let line_visible_lengths: Vec<usize> = lines.iter().map(|line| visible_len(line)).collect();

    // Find the widest content line
    let content_width = line_visible_lengths.iter().copied().max().unwrap_or(0);

    // Title length - use chars().count() for Unicode correctness
    let title_char_count = title.map_or(0, |title_text| title_text.chars().count());

    // Box must be wide enough for both content AND title
    let minimum_width = content_width.max(title_char_count);
    let box_inner_width = target_width.unwrap_or(minimum_width).max(minimum_width);

    // Calculate height: content lines + 2 for top/bottom borders
    let content_line_count = lines.len();
    let minimum_height = content_line_count + 2;
    let box_total_height = target_height.unwrap_or(minimum_height).max(minimum_height);

    // --- step 3: Calculate vertical padding ---
    // Extra vertical space is split between top and bottom
    let total_vertical_padding = box_total_height.saturating_sub(minimum_height);
    let top_padding_rows = total_vertical_padding / 2;
    let bottom_padding_rows = total_vertical_padding - top_padding_rows;

    let mut result = Vec::with_capacity(box_total_height);

    // --- step 4: Pre-compute reusable colored border pieces ---
    let colored_vertical_border = color_border(box_vertical);
    let colored_horizontal_line = color_border(&box_horizontal.repeat(box_inner_width + 2));
    let inner_spaces = " ".repeat(box_inner_width + 2);
    let empty_padding_row = format!("{colored_vertical_border}{inner_spaces}{colored_vertical_border}");

    // --- step 5: Build top border ---
    // Format: ╭──── Title ────╮  or  ╭────────────╮
    let top_border = if let Some(title_text) = title {
        // Calculate dashes on each side of the title
        let total_dash_count = box_inner_width.saturating_sub(title_char_count);
        let left_dash_count = total_dash_count / 2;
        let right_dash_count = total_dash_count - left_dash_count;
        format!(
            "{}{} {} {}{}",
            color_border(box_top_left),
            color_border(&box_horizontal.repeat(left_dash_count)),
            color_title(title_text),
            color_border(&box_horizontal.repeat(right_dash_count)),
            color_border(box_top_right)
        )
    } else {
        // No title - just a solid horizontal line
        format!(
            "{}{}{}",
            color_border(box_top_left),
            colored_horizontal_line,
            color_border(box_top_right)
        )
    };
    result.push(top_border);

    // --- step 6: Add top padding rows ---
    for _ in 0..top_padding_rows {
        result.push(empty_padding_row.clone());
    }

    // --- step 7: Build content rows ---
    // Format: │ [left_pad] content [right_pad] │
    // For center_content, we center the entire content block as a unit (preserving relative positions)
    // rather than centering each line independently
    let block_left_padding = if center_content {
        let max_line_width = line_visible_lengths.iter().copied().max().unwrap_or(0);
        (box_inner_width.saturating_sub(max_line_width)) / 2
    } else {
        0
    };

    for (line_content, &line_visible_width) in lines.iter().zip(line_visible_lengths.iter()) {
        // Right padding fills remaining space after block padding and content
        let right_padding_spaces = box_inner_width.saturating_sub(block_left_padding + line_visible_width);

        let content_row = format!(
            "{} {}{}{} {}",
            colored_vertical_border,
            " ".repeat(block_left_padding),
            line_content,
            " ".repeat(right_padding_spaces),
            colored_vertical_border
        );
        result.push(content_row);
    }

    // --- step 8: Add bottom padding rows ---
    for _ in 0..bottom_padding_rows {
        result.push(empty_padding_row.clone());
    }

    // --- step 9: Build bottom border ---
    let bottom_border = format!(
        "{}{}{}",
        color_border(box_bottom_left),
        colored_horizontal_line,
        color_border(box_bottom_right)
    );
    result.push(bottom_border);

    result
}

// Convert sections into formatted, boxed output lines.
//
// All boxes are given the same width for visual consistency.
pub fn build_sections_lines(sections: &[Section], target_width: Option<usize>) -> Vec<String> {
    // ---step 1: Format all key-value pairs with colors ---
    let formatted_sections: Vec<Vec<String>> = sections
        .iter()
        .map(|section| {
            section
                .lines
                .iter()
                .map(|(key, value)| {
                    if value.is_empty() {
                        // Key-only line with colon (e.g., "Display:")
                        format!("{}:", color_key(key))
                    } else if key.starts_with('├') || key.starts_with('╰') {
                        // Tree branch entries (no colon)
                        format!("{} {}", color_key(key), color_value(value))
                    } else {
                        format!("{}: {}", color_key(key), color_value(value))
                    }
                })
                .collect()
        })
        .collect();

    // ---step 2: Calculate the maximum content width across all sections ---
    // Need to consider both titles and formatted content lines
    let max_content_width = sections
        .iter()
        .zip(formatted_sections.iter())
        .flat_map(|(section, formatted_lines)| {
            // Include title width and all content line widths
            std::iter::once(section.title.chars().count())
                .chain(formatted_lines.iter().map(|line| visible_len(line)))
        })
        .max()
        .unwrap_or(0);

    // Use target width if larger, otherwise use calculated width
    let unified_box_width = target_width.unwrap_or(max_content_width).max(max_content_width);

    // === STEP 3: Build boxes for each section and combine ===
    let mut result = Vec::new();
    for (section_index, section) in sections.iter().enumerate() {
        let section_box = build_box(
            &formatted_sections[section_index],
            Some(&section.title),
            Some(unified_box_width),
            None,
            false, // Left-aligned content
        );
        result.extend(section_box);
    }

    result
}

// Calculate the maximum visible width of ASCII art lines.
#[inline]
fn art_width(art: &[String]) -> usize {
    art.iter().map(|line| visible_len(line)).max().unwrap_or(0)
}

// Render two boxes side-by-side (art on left, sections on right).
//
// Handles cases where boxes have different heights by padding the shorter one.
fn render_side_by_side(art_box: &[String], sections_box: &[String], output: &mut String) {
    let total_row_count = art_box.len().max(sections_box.len());

    // Pre-compute padding for when art_box runs out of lines
    let art_box_visual_width = art_box.first().map(|first_line| visible_len(first_line)).unwrap_or(0);
    let art_padding_spaces = " ".repeat(art_box_visual_width);

    // Build each row: [art_line or padding] [space] [section_line]
    for row_index in 0..total_row_count {
        // Left side: art box (or padding if we've run out of art lines)
        if row_index < art_box.len() {
            output.push_str(&art_box[row_index]);
        } else {
            output.push_str(&art_padding_spaces);
        }

        // Gap between boxes
        output.push(' ');

        // Right side: sections box
        if row_index < sections_box.len() {
            output.push_str(&sections_box[row_index]);
        }

        output.push('\n');
    }
}

// Render two boxes stacked vertically (art on top, sections below)
fn render_stacked(art_box: &[String], sections_box: &[String], output: &mut String) {
    // Art box first (on top)
    for line in art_box {
        output.push_str(line);
        output.push('\n');
    }
    // Sections box below
    for line in sections_box {
        output.push_str(line);
        output.push('\n');
    }
}

// Draw ASCII art and system info sections with adaptive layout.
//
// Layout selection priority (based on terminal dimensions):
// 1. Wide art side-by-side (big rig)
// 2. Smol art side-by-side
// 3. Smol art stacked (if terminal is tall enough but not wide enough)
// 4. Narrow art stacked (default stacked layout)
// 5. Sections only (if terminal is too small for any art)
pub fn draw_layout(
    wide_art: &[String],
    narrow_art: &[String],
    sections: &[Section],
    smol_art: Option<&[String]>,
) -> String {
    // ---step 1: Calculate all art widths ---
    let wide_art_width = art_width(wide_art);
    let narrow_art_width = art_width(narrow_art);
    let smol_art_width = smol_art.map(art_width).unwrap_or(0);

    // ---step 2: Calculate sections width ---
    // Each line is "Key: Value", so width = key_len + 2 (": ") + value_len
    let sections_content_width = sections
        .iter()
        .flat_map(|section| {
            std::iter::once(section.title.chars().count())
                .chain(section.lines.iter().map(|(key, value)| {
                    visible_len(key) + 2 + visible_len(value)
                }))
        })
        .max()
        .unwrap_or(0);

    // ---step 3: Calculate total widths for side-by-side layouts ---
    // Box width = content + 4 (2 for borders, 2 for internal margins)
    // Side-by-side = art_box + 1 (gap) + sections_box
    let sections_box_width = sections_content_width + 4;
    let wide_side_by_side_width = wide_art_width + 4 + 1 + sections_box_width;
    let smol_side_by_side_width = smol_art_width + 4 + 1 + sections_box_width;

    // ---step 4: Get terminal dimensions ---
    let (terminal_width, terminal_height) = get_terminal_size()
        .map(|(cols, rows)| (cols as usize, rows as usize))
        .unwrap_or((80, 24)); // Fallback to standard 80x24 terminal

    // ---step 5: Calculate heights for stacked layouts ---
    // Sections height = sum of (content lines + 2 borders) for each section
    let sections_total_height: usize = sections
        .iter()
        .map(|section| section.lines.len() + 2)
        .sum();
    let narrow_art_box_height = narrow_art.len() + 2;

    // ---step 6: Select layout based on terminal size ---
    let mut output = String::new();

    if terminal_width >= wide_side_by_side_width {
        // layout 1: Wide art side-by-side
        let sections_box = build_sections_lines(sections, None);
        // If sections are shorter than wide art, use narrow art instead
        let wide_art_box_height = wide_art.len() + 2;
        if sections_box.len() < wide_art_box_height {
            // Fall back to narrow art side-by-side
            let narrow_side_by_side_width = narrow_art_width + 4 + 1 + sections_box_width;
            if terminal_width >= narrow_side_by_side_width {
                let art_box = build_box(narrow_art, None, None, Some(sections_box.len()), true);
                render_side_by_side(&art_box, &sections_box, &mut output);
            } else {
                // Terminal too narrow for narrow art side-by-side, just use sections
                for line in &sections_box {
                    output.push_str(line);
                    output.push('\n');
                }
            }
        } else {
            let art_box = build_box(wide_art, None, None, Some(sections_box.len()), true);
            render_side_by_side(&art_box, &sections_box, &mut output);
        }
    } else if smol_art.is_some() && terminal_width >= smol_side_by_side_width {
        // layout 2: Smol art side-by-side
        let smol_art_lines = smol_art.unwrap();
        let sections_box = build_sections_lines(sections, None);
        let smol_art_box_height = smol_art_lines.len() + 2;
        // If sections are shorter than smol art, fall back to narrow art
        if sections_box.len() < smol_art_box_height {
            let narrow_side_by_side_width = narrow_art_width + 4 + 1 + sections_box_width;
            if terminal_width >= narrow_side_by_side_width {
                let art_box = build_box(narrow_art, None, None, Some(sections_box.len()), true);
                render_side_by_side(&art_box, &sections_box, &mut output);
            } else {
                for line in &sections_box {
                    output.push_str(line);
                    output.push('\n');
                }
            }
        } else {
            let art_box = build_box(smol_art_lines, None, None, Some(sections_box.len()), true);
            render_side_by_side(&art_box, &sections_box, &mut output);
        }
    } else if smol_art.is_some() && terminal_height >= sections_total_height + smol_art.unwrap().len() + 2 {
        // layout 3: Smol art stacked
        let smol_art_lines = smol_art.unwrap();
        let stacked_width = smol_art_width.max(sections_content_width);
        let art_box = build_box(smol_art_lines, None, Some(stacked_width), None, true);
        let sections_box = build_sections_lines(sections, Some(stacked_width));
        render_stacked(&art_box, &sections_box, &mut output);
    } else if terminal_height >= sections_total_height + narrow_art_box_height {
        // layout 4: Narrow art stacked
        let stacked_width = narrow_art_width.max(sections_content_width);
        let art_box = build_box(narrow_art, None, Some(stacked_width), None, true);
        let sections_box = build_sections_lines(sections, Some(stacked_width));
        render_stacked(&art_box, &sections_box, &mut output);
    } else {
        // layout 5: Sections only
        let sections_box = build_sections_lines(sections, None);
        for line in &sections_box {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}
