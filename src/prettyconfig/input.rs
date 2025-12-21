// Input handling for prettyconfig TUI
// Key event processing and user interaction

use crossterm::event::KeyCode;

use crate::prettyconfig::helpers::{next_theme, prev_theme};
use crate::prettyconfig::navigation::{App, FocusArea};
use crate::prettyconfig::save;

impl App {
    pub fn handle_key(&mut self, key: KeyCode) {
        if self.editing {
            self.handle_editing_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('s') => self.save_config(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            KeyCode::Char(' ') | KeyCode::Enter => self.handle_select(),
            KeyCode::Tab => self.next_focus(),
            KeyCode::BackTab => self.prev_focus(),
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.editing = false;
                self.edit_buffer.clear();
            }
            KeyCode::Enter => self.finish_editing(),
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.edit_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.edit_buffer.len() {
                    self.edit_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.edit_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => self.cursor_pos = 0,
            KeyCode::End => self.cursor_pos = self.edit_buffer.len(),
            KeyCode::Char(c) => {
                self.edit_buffer.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        match self.focus {
            FocusArea::Art => match self.index {
                0 => {
                    self.theme = prev_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_os_art_prev();
                    self.update_preview();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_right(&mut self) {
        match self.focus {
            FocusArea::Art => match self.index {
                0 => {
                    self.theme = next_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_os_art_next();
                    self.update_preview();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_select(&mut self) {
        match self.focus {
            FocusArea::Art => match self.index {
                0 => {
                    self.theme = next_theme(self.theme);
                    self.update_preview();
                }
                1 => {
                    self.cycle_os_art_next();
                    self.update_preview();
                }
                2 => self.start_editing(self.custom_art.clone().unwrap_or_default()),
                _ => {}
            },
            FocusArea::Image => match self.index {
                0 => {
                    self.image = !self.image;
                    self.update_image_protocol();
                    self.update_preview();
                }
                1 => self.start_editing(self.image_path.clone().unwrap_or_default()),
                _ => {}
            },
            FocusArea::Core => {
                match self.index {
                    0 => self.core.os = !self.core.os,
                    1 => self.core.kernel = !self.core.kernel,
                    2 => self.core.uptime = !self.core.uptime,
                    _ => {}
                }
                self.update_preview();
            }
            FocusArea::Hardware => {
                match self.index {
                    0 => self.hardware.cpu = !self.hardware.cpu,
                    1 => self.hardware.gpu = !self.hardware.gpu,
                    2 => self.hardware.memory = !self.hardware.memory,
                    3 => self.hardware.storage = !self.hardware.storage,
                    4 => self.hardware.battery = !self.hardware.battery,
                    5 => self.hardware.screen = !self.hardware.screen,
                    _ => {}
                }
                self.update_preview();
            }
            FocusArea::Userspace => {
                match self.index {
                    0 => self.userspace.packages = !self.userspace.packages,
                    1 => self.userspace.terminal = !self.userspace.terminal,
                    2 => self.userspace.shell = !self.userspace.shell,
                    3 => self.userspace.wm = !self.userspace.wm,
                    4 => self.userspace.ui = !self.userspace.ui,
                    5 => self.userspace.editor = !self.userspace.editor,
                    6 => self.userspace.terminal_font = !self.userspace.terminal_font,
                    _ => {}
                }
                self.update_preview();
            }
            FocusArea::Buttons => match self.index {
                0 => self.save_config(),
                1 => self.should_exit = true,
                _ => {}
            },
        }
    }

    pub fn save_config(&mut self) {
        match save::save_config(
            self.theme,
            &self.os_art,
            &self.custom_art,
            self.image,
            &self.image_path,
            &self.core,
            &self.hardware,
            &self.userspace,
        ) {
            Ok(path) => {
                self.status_message = Some(format!("Saved to {:?}", path));
                self.should_exit = true;
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }
}
