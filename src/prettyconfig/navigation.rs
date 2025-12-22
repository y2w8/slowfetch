// Navigation and state management for prettyconfig TUI
// Contains FocusArea enum, App struct, and navigation methods

use crate::configloader::{
    Config, CoreToggles, HardwareToggles, OsArtSetting, ThemePreset, UserspaceToggles,
};
use crate::dostuff;
use crate::modules::asciimodule;
use crate::prettyconfig::helpers::{theme_color_to_ratatui};
use crate::prettyconfig::preview::detect_theme_from_config;
use crate::visuals::renderer::Section;

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

// Focus areas for Tab navigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusArea {
    Art,       // Theme, OS Art, Custom Art path
    Image,     // Image toggle, Image path
    Core,      // Core toggles
    Hardware,  // Hardware toggles
    Userspace, // Userspace toggles
    Buttons,   // Save/Cancel
}

impl FocusArea {
    pub fn next(self) -> Self {
        match self {
            Self::Art => Self::Image,
            Self::Image => Self::Core,
            Self::Core => Self::Hardware,
            Self::Hardware => Self::Userspace,
            Self::Userspace => Self::Buttons,
            Self::Buttons => Self::Art,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Art => Self::Buttons,
            Self::Image => Self::Art,
            Self::Core => Self::Image,
            Self::Hardware => Self::Core,
            Self::Userspace => Self::Hardware,
            Self::Buttons => Self::Userspace,
        }
    }

    pub fn max_index(self) -> usize {
        match self {
            Self::Art => 2,       // Theme, OS Art, Custom Art
            Self::Image => 1,     // Enabled, Path
            Self::Core => 2,      // OS, Kernel, Uptime
            Self::Hardware => 5,  // CPU, GPU, Memory, Storage, Battery, Screen
            Self::Userspace => 6, // Packages, Terminal, Shell, WM, UI, Editor, Term Font
            Self::Buttons => 1,   // Save, Cancel
        }
    }
}

// Application state
pub struct App {
    // Config values
    pub theme: ThemePreset,
    pub os_art: OsArtSetting,
    pub custom_art: Option<String>,
    pub image: bool,
    pub image_path: Option<String>,
    pub core: CoreToggles,
    pub hardware: HardwareToggles,
    pub userspace: UserspaceToggles,

    // Navigation - Tab switches focus area, Up/Down selects within area
    pub focus: FocusArea,
    pub index: usize,

    // Text editing
    pub editing: bool,
    pub edit_buffer: String,
    pub cursor_pos: usize,

    // Cached preview data
    pub preview_lines: Vec<String>,
    pub sections_only_lines: Vec<String>, // For image mode: just the sections without art
    pub cached_sections: (Section, Section, Section),

    // Image preview state
    pub picker: Picker,
    pub image_protocol: Option<StatefulProtocol>,

    // Status message
    pub status_message: Option<String>,

    // Exit flag
    pub should_exit: bool,

    // Layout regions for mouse hit-testing
    pub layout: LayoutRegions,
}

// Cached layout regions for mouse click detection
#[derive(Default, Clone)]
pub struct LayoutRegions {
    pub art_box: Rect,
    pub image_box: Rect,
    pub core_box: Rect,
    pub hardware_box: Rect,
    pub userspace_box: Rect,
    pub save_button: Rect,
    pub cancel_button: Rect,
}

impl App {
    pub fn from_config(config: &Config) -> Self {
        let theme = detect_theme_from_config(config);

        // Load sections with ALL toggles enabled for preview
        let mut full_config = config.clone();
        full_config.core = CoreToggles { os: true, kernel: true, uptime: true };
        full_config.hardware = HardwareToggles {
            cpu: true, gpu: true, memory: true, storage: true, battery: true, screen: true,
        };
        full_config.userspace = UserspaceToggles {
            packages: true, terminal: true, shell: true, wm: true, ui: true, editor: true, terminal_font: true,
        };
        let sections = dostuff::load_sections(&full_config);

        // Initialize image picker - query terminal for graphics protocol support
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
            // Fallback to a reasonable default font size
            Picker::from_fontsize((8, 16))
        });

        let mut app = Self {
            theme,
            os_art: config.os_art.clone(),
            custom_art: config.custom_art.clone(),
            image: config.image,
            image_path: config.image_path.clone(),
            core: config.core.clone(),
            hardware: config.hardware.clone(),
            userspace: config.userspace.clone(),

            focus: FocusArea::Art,
            index: 0,

            editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,

            preview_lines: Vec::new(),
            sections_only_lines: Vec::new(),
            cached_sections: sections,

            picker,
            image_protocol: None,

            status_message: None,
            should_exit: false,

            layout: LayoutRegions::default(),
        };

        app.update_preview();
        app.update_image_protocol();
        app
    }

    pub fn update_image_protocol(&mut self) {
        if !self.image {
            self.image_protocol = None;
            return;
        }

        // Try to load the image
        let img_result = if let Some(ref path) = self.image_path {
            image::ImageReader::open(path)
                .ok()
                .and_then(|reader| reader.decode().ok())
        } else {
            // Load embedded default image
            image::load_from_memory(include_bytes!("../assets/default/slowfetch.png")).ok()
        };

        self.image_protocol = img_result.map(|img| self.picker.new_resize_protocol(img));
    }

    pub fn colors(&self) -> (Color, Color, Color, Color) {
        let (border, title, key, value) = self.theme.colors();
        (
            theme_color_to_ratatui(border),
            theme_color_to_ratatui(title),
            theme_color_to_ratatui(key),
            theme_color_to_ratatui(value),
        )
    }

    pub fn get_art_for_preview(&self) -> (Vec<String>, Vec<String>, Option<Vec<String>>) {
        let wide_logo = asciimodule::get_wide_logo_lines();
        let narrow_logo = asciimodule::get_narrow_logo_lines();

        if let Some(ref custom_path) = self.custom_art {
            if let Some(custom_art) = asciimodule::get_custom_art_lines(custom_path) {
                return (custom_art.clone(), custom_art, None);
            }
        }

        let os_name: String = self.cached_sections.0
            .lines
            .iter()
            .find(|(k, _)| k == "OS")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        match &self.os_art {
            OsArtSetting::Disabled => (wide_logo, narrow_logo, None),
            OsArtSetting::Auto => {
                if let Some(os_logo) = asciimodule::get_os_logo_lines(&os_name) {
                    let smol_logo = asciimodule::get_os_logo_lines_smol(&os_name);
                    (os_logo.clone(), os_logo, smol_logo)
                } else {
                    (wide_logo, narrow_logo, None)
                }
            }
            OsArtSetting::Specific(specific_os) => {
                if let Some(os_logo) = asciimodule::get_os_logo_lines(specific_os) {
                    let smol_logo = asciimodule::get_os_logo_lines_smol(specific_os);
                    (os_logo.clone(), os_logo, smol_logo)
                } else {
                    (wide_logo, narrow_logo, None)
                }
            }
        }
    }

    // Navigation methods
    pub fn next_focus(&mut self) {
        self.focus = self.focus.next();
        self.index = 0;
    }

    pub fn prev_focus(&mut self) {
        self.focus = self.focus.prev();
        self.index = 0;
    }

    pub fn move_up(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.index < self.focus.max_index() {
            self.index += 1;
        }
    }

    pub fn cycle_os_art_next(&mut self) {
        self.os_art = match &self.os_art {
            OsArtSetting::Disabled => OsArtSetting::Auto,
            OsArtSetting::Auto => OsArtSetting::Disabled,
            OsArtSetting::Specific(_) => OsArtSetting::Disabled,
        };
    }

    pub fn cycle_os_art_prev(&mut self) {
        self.os_art = match &self.os_art {
            OsArtSetting::Disabled => OsArtSetting::Auto,
            OsArtSetting::Auto => OsArtSetting::Disabled,
            OsArtSetting::Specific(_) => OsArtSetting::Auto,
        };
    }

    // Text editing methods
    pub fn start_editing(&mut self, initial: String) {
        self.editing = true;
        self.edit_buffer = initial;
        self.cursor_pos = self.edit_buffer.len();
    }

    pub fn finish_editing(&mut self) {
        self.editing = false;
        let value = if self.edit_buffer.is_empty() {
            None
        } else {
            Some(self.edit_buffer.clone())
        };

        match self.focus {
            FocusArea::Art if self.index == 2 => {
                self.custom_art = value;
                self.update_preview();
            }
            FocusArea::Image if self.index == 1 => {
                self.image_path = value;
                self.update_image_protocol();
                self.update_preview();
            }
            _ => {}
        }

        self.edit_buffer.clear();
    }
}

// Import preview module for update_preview
use crate::prettyconfig::preview;

impl App {
    pub fn update_preview(&mut self) {
        preview::update_preview(self);
    }
}
