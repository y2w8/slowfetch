// Prettyconfig - TUI configuration editor for Slowfetch
// Modular structure for maintainability

mod helpers;
mod input;
mod navigation;
mod prettyconfig;
mod preview;
mod render;
mod save;

pub use prettyconfig::run;
