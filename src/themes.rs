// Theme presets for Slowfetch
// Provides color schemes for UI elements and ASCII art

use crate::configloader::ThemeColor;

// Theme presets for theme colors (border, title, key, value)
// Default uses None to indicate "use terminal's default colors"
#[derive(Debug, Clone, Copy, Default)]
pub enum ThemePreset {
    #[default]
    Default,
    Dracula,
    Catppuccin,
    Nord,
    Gruvbox,
    Eldritch,
    Kanagawa,
    RosePine,
}

impl ThemePreset {
    // Parse theme name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" | "terminal" | "tty" => Some(Self::Default),
            "dracula" => Some(Self::Dracula),
            "catppuccin" | "cat" => Some(Self::Catppuccin),
            "nord" => Some(Self::Nord),
            "gruvbox" | "gruv" => Some(Self::Gruvbox),
            "eldritch" => Some(Self::Eldritch),
            "kanagawa" | "kana" => Some(Self::Kanagawa),
            "rosepine" | "rose-pine" | "rose_pine" => Some(Self::RosePine),
            _ => None,
        }
    }

    // Get UI theme colors: (border, title, key, value)
    pub fn colors(self) -> (ThemeColor, ThemeColor, ThemeColor, ThemeColor) {
        match self {
            // Default: ANSI 1 (red), 3 (yellow), 4 (blue), 5 (magenta)
            Self::Default => (
                ThemeColor::Ansi(4), // border
                ThemeColor::Ansi(5), // title
                ThemeColor::Ansi(5), // key
                ThemeColor::Ansi(6), // value
            ),
            Self::Dracula => (
                ThemeColor::Rgb(0xFF, 0x79, 0xC6), // border: #FF79C6 - pink
                ThemeColor::Rgb(0xFF, 0x79, 0xC6), // title:  #FF79C6 - pink
                ThemeColor::Rgb(0xBD, 0x93, 0xF9), // key:    #BD93F9 - purple
                ThemeColor::Rgb(0x8B, 0xE9, 0xFD), // value:  #8BE9FD - cyan
            ),
            Self::Catppuccin => (
                // Catppuccin Mocha - https://catppuccin.com/palette
                ThemeColor::Rgb(0xF3, 0x8B, 0xA8), // border: #F5C2E7 - pinky red
                ThemeColor::Rgb(0xCB, 0xA6, 0xF7), // title:  #CBA6F7 - mauve
                ThemeColor::Rgb(0x89, 0xB4, 0xFA), // key:    #89B4FA - blue
                ThemeColor::Rgb(0xF5, 0xC2, 0xE7), // value:  #F38BA8 - pink
            ),
            Self::Nord => (
                // Nord - https://nordtheme.com/docs/colors-and-palettes
                ThemeColor::Rgb(0xB4, 0x8E, 0xAD), // border: #B48EAD - nord15 purple
                ThemeColor::Rgb(0x88, 0xC0, 0xD0), // title:  #88C0D0 - nord8 frost cyan
                ThemeColor::Rgb(0x81, 0xA1, 0xC1), // key:    #81A1C1 - nord9 frost blue
                ThemeColor::Rgb(0x5E, 0x81, 0xAC), // value:  #5E81AC - nord10 frost dark blue
            ),
            Self::Gruvbox => (
                // Gruvbox Dark - https://github.com/morhetz/gruvbox
                ThemeColor::Rgb(0xB8, 0xBB, 0x26), // border: #B8BB26 - bright green
                ThemeColor::Rgb(0xFB, 0x49, 0x34), // title:  #FB4934 - bright red
                ThemeColor::Rgb(0xFA, 0xBD, 0x2F), // key:    #FABD2F - bright yellow
                ThemeColor::Rgb(0x83, 0xA5, 0x98), // value:  #83A598 - bright blue
            ),
            Self::Eldritch => (
                // Eldritch - https://github.com/eldritch-theme/eldritch
                ThemeColor::Rgb(0xF2, 0x65, 0xB5), // border: #F265B5 - pink
                ThemeColor::Rgb(0xA4, 0x8C, 0xF2), // title:  #A48CF2 - purple
                ThemeColor::Rgb(0x37, 0xF4, 0x99), // key:    #37F499 - green
                ThemeColor::Rgb(0x04, 0xD1, 0xF9), // value:  #04D1F9 - cyan
            ),
            Self::Kanagawa => (
                // Kanagawa Wave - https://github.com/rebelot/kanagawa.nvim
                ThemeColor::Rgb(0xC0, 0xA3, 0x6E), // border: #76946A - boatYellow2
                ThemeColor::Rgb(0x76, 0x94, 0x6A), // title:  #C0A36E - autumnGreen
                ThemeColor::Rgb(0x7E, 0x9C, 0xD8), // key:    #7E9CD8 - crystalBlue
                ThemeColor::Rgb(0x98, 0xBB, 0x6C), // value:  #98BB6C - springGreen
            ),
            Self::RosePine => (
                // Rosé Pine - https://rosepinetheme.com/palette
                ThemeColor::Rgb(0xC4, 0xA7, 0xE7), // border: #C4A7E7 - iris
                ThemeColor::Rgb(0xEB, 0xBB, 0xBA), // title:  #EBBCBA - rose
                ThemeColor::Rgb(0x31, 0x74, 0x8F), // key:    #31748F - pine
                ThemeColor::Rgb(0x9C, 0xCF, 0xD8), // value:  #9CCFD8 - foam
            ),
        }
    }

    // Get art colors for the theme: [art_1 through art_8]
    // Returns None for Default theme (uses terminal ANSI colors)
    // All other themes use truecolor RGB values
    pub fn art_colors(self) -> Option<[ThemeColor; 8]> {
        match self {
            Self::Default => None, // Use terminal ANSI colors only for default
            Self::Dracula => Some([
                ThemeColor::Rgb(0xFF, 0x55, 0x55), // {0} red
                ThemeColor::Rgb(0xFF, 0xB8, 0x6C), // {1} orange
                ThemeColor::Rgb(0xF1, 0xFA, 0x8C), // {2} yellow
                ThemeColor::Rgb(0x50, 0xFA, 0x7B), // {3} green
                ThemeColor::Rgb(0x8B, 0xE9, 0xFD), // {4} cyan
                ThemeColor::Rgb(0xBD, 0x93, 0xF9), // {5} purple
                ThemeColor::Rgb(0xFF, 0x79, 0xC6), // {6} pink
                ThemeColor::Rgb(0x6E, 0x46, 0x6B), // {7} dark purple
            ]),
            Self::Catppuccin => Some([
                // Catppuccin Mocha palette
                ThemeColor::Rgb(0xF3, 0x8B, 0xA8), // {0} red
                ThemeColor::Rgb(0xFA, 0xB3, 0x87), // {1} peach
                ThemeColor::Rgb(0xF9, 0xE2, 0xAF), // {2} yellow
                ThemeColor::Rgb(0xA6, 0xE3, 0xA1), // {3} green
                ThemeColor::Rgb(0x94, 0xE2, 0xD5), // {4} teal
                ThemeColor::Rgb(0x89, 0xB4, 0xFA), // {5} blue
                ThemeColor::Rgb(0xCB, 0xA6, 0xF7), // {6} mauve
                ThemeColor::Rgb(0xF5, 0xC2, 0xE7), // {7} pink
            ]),
            Self::Nord => Some([
                // Nord palette - aurora + frost
                ThemeColor::Rgb(0xBF, 0x61, 0x6A), // {0} nord11 red
                ThemeColor::Rgb(0xD0, 0x87, 0x70), // {1} nord12 orange
                ThemeColor::Rgb(0xEB, 0xCB, 0x8B), // {2} nord13 yellow
                ThemeColor::Rgb(0xA3, 0xBE, 0x8C), // {3} nord14 green
                ThemeColor::Rgb(0x88, 0xC0, 0xD0), // {4} nord8 cyan
                ThemeColor::Rgb(0x81, 0xA1, 0xC1), // {5} nord9 blue
                ThemeColor::Rgb(0xB4, 0x8E, 0xAD), // {6} nord15 purple
                ThemeColor::Rgb(0x5E, 0x81, 0xAC), // {7} nord10 dark blue
            ]),
            Self::Gruvbox => Some([
                // Gruvbox Dark palette
                ThemeColor::Rgb(0xFB, 0x49, 0x34), // {0} bright red
                ThemeColor::Rgb(0xFE, 0x80, 0x19), // {1} bright orange
                ThemeColor::Rgb(0xFA, 0xBD, 0x2F), // {2} bright yellow
                ThemeColor::Rgb(0xB8, 0xBB, 0x26), // {3} bright green
                ThemeColor::Rgb(0x8E, 0xC0, 0x7C), // {4} bright aqua
                ThemeColor::Rgb(0x83, 0xA5, 0x98), // {5} bright blue
                ThemeColor::Rgb(0xD3, 0x86, 0x9B), // {6} bright purple
                ThemeColor::Rgb(0xD6, 0x5D, 0x0E), // {7} dark orange
            ]),
            Self::Eldritch => Some([
                // Eldritch palette
                ThemeColor::Rgb(0xF1, 0x6C, 0x75), // {0} red
                ThemeColor::Rgb(0xF7, 0xC6, 0x7F), // {1} yellow/orange
                ThemeColor::Rgb(0xEB, 0xFF, 0x87), // {2} bright yellow
                ThemeColor::Rgb(0x37, 0xF4, 0x99), // {3} green
                ThemeColor::Rgb(0x04, 0xD1, 0xF9), // {4} cyan
                ThemeColor::Rgb(0x7C, 0xB7, 0xFF), // {5} blue
                ThemeColor::Rgb(0xA4, 0x8C, 0xF2), // {6} purple
                ThemeColor::Rgb(0xF2, 0x65, 0xB5), // {7} pink
            ]),
            Self::Kanagawa => Some([
                // Kanagawa Wave palette
                ThemeColor::Rgb(0xC3, 0x40, 0x43), // {0} autumnRed
                ThemeColor::Rgb(0xFF, 0xA0, 0x66), // {1} surimiOrange
                ThemeColor::Rgb(0xC0, 0xA3, 0x6E), // {2} boatYellow2
                ThemeColor::Rgb(0x76, 0x94, 0x6A), // {3} autumnGreen
                ThemeColor::Rgb(0x7F, 0xB4, 0xCA), // {4} springBlue
                ThemeColor::Rgb(0x7E, 0x9C, 0xD8), // {5} crystalBlue
                ThemeColor::Rgb(0x95, 0x7F, 0xB8), // {6} oniViolet
                ThemeColor::Rgb(0xD2, 0x7E, 0x99), // {7} sakuraPink
            ]),
            Self::RosePine => Some([
                // Rosé Pine palette
                ThemeColor::Rgb(0xEB, 0x6F, 0x92), // {0} love (red/pink)
                ThemeColor::Rgb(0xF6, 0xC1, 0x77), // {1} gold (yellow/orange)
                ThemeColor::Rgb(0xEA, 0x9A, 0x97), // {2} rose (soft pink)
                ThemeColor::Rgb(0x31, 0x74, 0x8F), // {3} pine (teal/green)
                ThemeColor::Rgb(0x9C, 0xCF, 0xD8), // {4} foam (cyan)
                ThemeColor::Rgb(0xC4, 0xA7, 0xE7), // {5} iris (purple)
                ThemeColor::Rgb(0xEB, 0xBB, 0xBA), // {6} rose (light pink)
                ThemeColor::Rgb(0x90, 0x8C, 0xAA), // {7} subtle (muted lavender)
            ]),
        }
    }
}
