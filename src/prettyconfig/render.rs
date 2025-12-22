// Rendering functions for prettyconfig TUI
// All draw_* functions and ANSI parsing utilities

use crate::configloader::OsArtSetting;
use crate::prettyconfig::helpers::theme_name;
use crate::prettyconfig::navigation::{App, FocusArea};

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui_image::StatefulImage;

/// Main draw function - renders the entire UI
pub fn draw(frame: &mut Frame, app: &mut App) {
    let (border_color, title_color, key_color, value_color) = app.colors();
    let area = frame.area();

    let main_chunks = Layout::vertical([
        Constraint::Length(19),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .split(area);

    draw_settings_panel(frame, app, main_chunks[0], border_color, title_color, key_color, value_color);
    draw_preview_panel(frame, app, main_chunks[1], border_color, title_color);
    draw_help_bar(frame, app, main_chunks[2]);
}

/// Draw the settings panel with art, image, and toggle sections
fn draw_settings_panel(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
    key_color: Color,
    value_color: Color,
) {
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Settings", Style::default().fg(title_color)),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Center);

    frame.render_widget(outer_block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let rows = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Min(9),
        Constraint::Length(1),
    ])
    .split(inner);

    let top_cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(rows[0]);

    draw_art_box(frame, app, top_cols[0], border_color, title_color, key_color, value_color);
    draw_image_box(frame, app, top_cols[1], border_color, title_color, key_color, value_color);
    draw_toggle_grid(frame, app, rows[2], border_color, title_color, key_color);
    draw_buttons(frame, app, rows[3], border_color, title_color);
}

/// Draw the art configuration box
fn draw_art_box(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
    key_color: Color,
    value_color: Color,
) {
    // Store region for mouse hit-testing
    app.layout.art_box = area;
    let focused = app.focus == FocusArea::Art;
    let box_border = if focused { title_color } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(box_border))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Art", Style::default().fg(title_color)),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Theme
    let selected = focused && app.index == 0;
    let style = if selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
    let line = Line::from(vec![
        Span::styled("Theme:      ", style.fg(key_color)),
        Span::styled(format!("◀ {:^12} ▶", theme_name(app.theme)), style.fg(value_color)),
    ]);
    frame.render_widget(Paragraph::new(line), Rect { y: inner.y, height: 1, ..inner });

    // OS Art
    let selected = focused && app.index == 1;
    let style = if selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
    let os_art_display = match &app.os_art {
        OsArtSetting::Disabled => "Disabled",
        OsArtSetting::Auto => "Auto",
        OsArtSetting::Specific(s) => s.as_str(),
    };
    let line = Line::from(vec![
        Span::styled("OS Art:     ", style.fg(key_color)),
        Span::styled(format!("◀ {:^12} ▶", os_art_display), style.fg(value_color)),
    ]);
    frame.render_widget(Paragraph::new(line), Rect { y: inner.y + 1, height: 1, ..inner });

    // Custom Art
    let selected = focused && app.index == 2;
    let style = if selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
    let value_width = inner.width.saturating_sub(14) as usize;
    let custom_value = if app.editing && selected {
        format_edit_buffer(&app.edit_buffer, app.cursor_pos)
    } else {
        app.custom_art.clone().unwrap_or_else(|| "(none)".to_string())
    };
    let display = truncate_path(&custom_value, value_width.saturating_sub(2));
    let line = Line::from(vec![
        Span::styled("Custom Art: ", style.fg(key_color)),
        Span::styled(format!("[{}]", display), style.fg(value_color)),
    ]);
    frame.render_widget(Paragraph::new(line), Rect { y: inner.y + 2, height: 1, ..inner });
}

/// Draw the image configuration box
fn draw_image_box(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
    key_color: Color,
    value_color: Color,
) {
    // Store region for mouse hit-testing
    app.layout.image_box = area;
    let focused = app.focus == FocusArea::Image;
    let box_border = if focused { title_color } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(box_border))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Image", Style::default().fg(title_color)),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Enabled toggle
    let selected = focused && app.index == 0;
    let style = if selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
    let checkbox = if app.image { "[x]" } else { "[ ]" };
    let line = Line::from(vec![
        Span::styled("Enabled:    ", style.fg(key_color)),
        Span::styled(checkbox, style.fg(value_color)),
    ]);
    frame.render_widget(Paragraph::new(line), Rect { y: inner.y, height: 1, ..inner });

    // Path
    let selected = focused && app.index == 1;
    let style = if selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
    let value_width = inner.width.saturating_sub(14) as usize;
    let path_value = if app.editing && selected {
        format_edit_buffer(&app.edit_buffer, app.cursor_pos)
    } else {
        app.image_path.clone().unwrap_or_else(|| "(none)".to_string())
    };
    let display = truncate_path(&path_value, value_width.saturating_sub(2));
    let line = Line::from(vec![
        Span::styled("Path:       ", style.fg(key_color)),
        Span::styled(format!("[{}]", display), style.fg(value_color)),
    ]);
    frame.render_widget(Paragraph::new(line), Rect { y: inner.y + 1, height: 1, ..inner });
}

/// Draw the toggle grid with Core, Hardware, and Userspace columns
fn draw_toggle_grid(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
    key_color: Color,
) {
    let cols = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ])
    .split(area);

    // Store regions for mouse hit-testing
    app.layout.core_box = cols[0];
    app.layout.hardware_box = cols[1];
    app.layout.userspace_box = cols[2];

    draw_toggle_column(frame, app, "Core", FocusArea::Core, &[
        ("OS", app.core.os),
        ("Kernel", app.core.kernel),
        ("Uptime", app.core.uptime),
    ], cols[0], border_color, title_color, key_color);

    draw_toggle_column(frame, app, "Hardware", FocusArea::Hardware, &[
        ("CPU", app.hardware.cpu),
        ("GPU", app.hardware.gpu),
        ("Memory", app.hardware.memory),
        ("Storage", app.hardware.storage),
        ("Battery", app.hardware.battery),
        ("Screen", app.hardware.screen),
    ], cols[1], border_color, title_color, key_color);

    draw_toggle_column(frame, app, "Userspace", FocusArea::Userspace, &[
        ("Packages", app.userspace.packages),
        ("Terminal", app.userspace.terminal),
        ("Shell", app.userspace.shell),
        ("WM", app.userspace.wm),
        ("UI", app.userspace.ui),
        ("Editor", app.userspace.editor),
        ("Term Font", app.userspace.terminal_font),
    ], cols[2], border_color, title_color, key_color);
}

/// Draw a single toggle column
fn draw_toggle_column(
    frame: &mut Frame,
    app: &App,
    title: &str,
    focus_area: FocusArea,
    items: &[(&str, bool)],
    area: Rect,
    border_color: Color,
    title_color: Color,
    key_color: Color,
) {
    let focused = app.focus == focus_area;
    let box_border = if focused { title_color } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(box_border))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(title, Style::default().fg(title_color)),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    for (i, (name, enabled)) in items.iter().enumerate() {
        if i >= inner.height as usize {
            break;
        }

        let selected = focused && app.index == i;
        let style = if selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let checkbox = if *enabled { "[x]" } else { "[ ]" };
        let line = Line::from(vec![
            Span::styled(format!("{} {}", checkbox, name), style.fg(key_color)),
        ]);

        frame.render_widget(Paragraph::new(line), Rect {
            x: inner.x,
            y: inner.y + i as u16,
            width: inner.width,
            height: 1,
        });
    }
}

/// Draw the Save/Cancel buttons
fn draw_buttons(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
) {
    let focused = app.focus == FocusArea::Buttons;
    let center_area = Rect {
        x: area.x + area.width / 2 - 15,
        y: area.y,
        width: 30,
        height: 1,
    };

    let cols = Layout::horizontal([
        Constraint::Length(12),
        Constraint::Length(2),
        Constraint::Length(12),
    ])
    .split(center_area);

    // Store regions for mouse hit-testing
    app.layout.save_button = cols[0];
    app.layout.cancel_button = cols[2];

    let save_selected = focused && app.index == 0;
    let save_style = if save_selected {
        Style::default().fg(title_color).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(border_color)
    };
    frame.render_widget(
        Paragraph::new("[ Save ]").style(save_style).alignment(Alignment::Center),
        cols[0],
    );

    let cancel_selected = focused && app.index == 1;
    let cancel_style = if cancel_selected {
        Style::default().fg(title_color).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(border_color)
    };
    frame.render_widget(
        Paragraph::new("[ Cancel ]").style(cancel_style).alignment(Alignment::Center),
        cols[2],
    );
}

/// Draw the preview panel with ASCII art or image
fn draw_preview_panel(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    border_color: Color,
    title_color: Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Preview", Style::default().fg(title_color)),
            Span::raw(" "),
        ]))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // If image mode is enabled and we have an image, render image + sections side by side
    if app.image {
        if let Some(ref mut protocol) = app.image_protocol {
            // Use sections-only lines (without ASCII art)
            let sections_height = app.sections_only_lines.len().min(inner.height as usize);
            let sections_width = app.sections_only_lines
                .iter()
                .map(|line| strip_ansi_width(line))
                .max()
                .unwrap_or(0);

            // Image box dimensions (including border) - make it square-ish based on sections height
            // Terminal cells are ~2:1 aspect ratio, so content width = height * 2
            // Add 2 for borders on each dimension
            let img_box_height = sections_height.min(inner.height as usize);
            let img_content_height = img_box_height.saturating_sub(2);
            let img_content_width = (img_content_height * 2).min(inner.width as usize / 2);
            let img_box_width = img_content_width + 2;

            // Total width: image box + gap + sections
            let total_width = img_box_width + 1 + sections_width;

            // Calculate offsets to center the whole layout
            let horizontal_offset = (inner.width as usize).saturating_sub(total_width) / 2;
            let vertical_offset = (inner.height as usize).saturating_sub(sections_height) / 2;

            // Render the image box (border) on the left
            let image_box_area = Rect {
                x: inner.x + horizontal_offset as u16,
                y: inner.y + vertical_offset as u16,
                width: img_box_width as u16,
                height: img_box_height as u16,
            };

            let image_box = Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(border_color));
            frame.render_widget(image_box, image_box_area);

            // Render the image inside the box
            let image_inner_area = Rect {
                x: image_box_area.x + 1,
                y: image_box_area.y + 1,
                width: img_content_width as u16,
                height: img_content_height as u16,
            };

            let image_widget = StatefulImage::default();
            frame.render_stateful_widget(image_widget, image_inner_area, protocol);

            // Render the sections text on the right
            let sections_area = Rect {
                x: inner.x + horizontal_offset as u16 + img_box_width as u16 + 1,
                y: inner.y + vertical_offset as u16,
                width: sections_width as u16,
                height: sections_height as u16,
            };

            let sections_text: Vec<Line> = app.sections_only_lines
                .iter()
                .take(sections_height)
                .map(|line| parse_ansi_to_line(line))
                .collect();

            frame.render_widget(Paragraph::new(sections_text), sections_area);
            return;
        }
    }

    // Fall back to text preview (ASCII art + sections)
    let content_height = app.preview_lines.len().min(inner.height as usize);
    let content_width = app.preview_lines
        .iter()
        .map(|line| strip_ansi_width(line))
        .max()
        .unwrap_or(0);

    let vertical_offset = (inner.height as usize).saturating_sub(content_height) / 2;
    let horizontal_offset = (inner.width as usize).saturating_sub(content_width) / 2;

    let preview_text: Vec<Line> = app.preview_lines
        .iter()
        .take(inner.height as usize)
        .map(|line| parse_ansi_to_line(line))
        .collect();

    let centered_area = Rect {
        x: inner.x + horizontal_offset as u16,
        y: inner.y + vertical_offset as u16,
        width: inner.width.saturating_sub(horizontal_offset as u16),
        height: inner.height.saturating_sub(vertical_offset as u16),
    };

    frame.render_widget(Paragraph::new(preview_text), centered_area);
}

/// Draw the help bar at the bottom
fn draw_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.editing {
        "Enter: Confirm | Esc: Cancel | Type to edit"
    } else {
        "Tab: Switch section | ↑↓: Select | ◀▶/Space: Change | s: Save | q: Quit"
    };

    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        area,
    );
}

// === ANSI Parsing Utilities ===

/// Calculate the display width of a string, stripping ANSI escape codes
pub fn strip_ansi_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == 'm' {
                        break;
                    }
                }
            }
        } else {
            width += 1;
        }
    }

    width
}

/// Parse an ANSI-colored string to a ratatui Line
fn parse_ansi_to_line(s: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();
    let mut current_text = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                chars.next();
                let mut sequence = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == 'm' {
                        chars.next();
                        break;
                    }
                    sequence.push(chars.next().unwrap());
                }
                current_style = parse_sgr_sequence(&sequence, current_style);
            }
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    Line::from(spans)
}

/// Parse SGR (Select Graphic Rendition) sequence
fn parse_sgr_sequence(sequence: &str, mut style: Style) -> Style {
    if sequence.is_empty() || sequence == "0" {
        return Style::default();
    }

    let codes: Vec<&str> = sequence.split(';').collect();
    let mut i = 0;

    while i < codes.len() {
        match codes[i] {
            "0" => style = Style::default(),
            "1" => style = style.add_modifier(Modifier::BOLD),
            "2" => style = style.add_modifier(Modifier::DIM),
            "3" => style = style.add_modifier(Modifier::ITALIC),
            "4" => style = style.add_modifier(Modifier::UNDERLINED),
            "7" => style = style.add_modifier(Modifier::REVERSED),
            "30" => style = style.fg(Color::Black),
            "31" => style = style.fg(Color::Red),
            "32" => style = style.fg(Color::Green),
            "33" => style = style.fg(Color::Yellow),
            "34" => style = style.fg(Color::Blue),
            "35" => style = style.fg(Color::Magenta),
            "36" => style = style.fg(Color::Cyan),
            "37" => style = style.fg(Color::White),
            "39" => style = style.fg(Color::Reset),
            "40" => style = style.bg(Color::Black),
            "41" => style = style.bg(Color::Red),
            "42" => style = style.bg(Color::Green),
            "43" => style = style.bg(Color::Yellow),
            "44" => style = style.bg(Color::Blue),
            "45" => style = style.bg(Color::Magenta),
            "46" => style = style.bg(Color::Cyan),
            "47" => style = style.bg(Color::White),
            "49" => style = style.bg(Color::Reset),
            "90" => style = style.fg(Color::DarkGray),
            "91" => style = style.fg(Color::LightRed),
            "92" => style = style.fg(Color::LightGreen),
            "93" => style = style.fg(Color::LightYellow),
            "94" => style = style.fg(Color::LightBlue),
            "95" => style = style.fg(Color::LightMagenta),
            "96" => style = style.fg(Color::LightCyan),
            "97" => style = style.fg(Color::White),
            "38" => {
                if i + 1 < codes.len() {
                    match codes[i + 1] {
                        "5" if i + 2 < codes.len() => {
                            if let Ok(n) = codes[i + 2].parse::<u8>() {
                                style = style.fg(Color::Indexed(n));
                            }
                            i += 2;
                        }
                        "2" if i + 4 < codes.len() => {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                codes[i + 2].parse::<u8>(),
                                codes[i + 3].parse::<u8>(),
                                codes[i + 4].parse::<u8>(),
                            ) {
                                style = style.fg(Color::Rgb(r, g, b));
                            }
                            i += 4;
                        }
                        _ => {}
                    }
                }
            }
            "48" => {
                if i + 1 < codes.len() {
                    match codes[i + 1] {
                        "5" if i + 2 < codes.len() => {
                            if let Ok(n) = codes[i + 2].parse::<u8>() {
                                style = style.bg(Color::Indexed(n));
                            }
                            i += 2;
                        }
                        "2" if i + 4 < codes.len() => {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                codes[i + 2].parse::<u8>(),
                                codes[i + 3].parse::<u8>(),
                                codes[i + 4].parse::<u8>(),
                            ) {
                                style = style.bg(Color::Rgb(r, g, b));
                            }
                            i += 4;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    style
}

// === Helper Functions ===

// Format edit buffer with cursor indicator
fn format_edit_buffer(buffer: &str, cursor_pos: usize) -> String {
    let mut result = String::with_capacity(buffer.len() + 1);
    for (i, c) in buffer.chars().enumerate() {
        if i == cursor_pos {
            result.push('│');
        }
        result.push(c);
    }
    if cursor_pos >= buffer.len() {
        result.push('│');
    }
    result
}

// Truncate path for display, showing end with ellipsis
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        format!("{:width$}", path, width = max_len)
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len.saturating_sub(3))..])
    }
}
