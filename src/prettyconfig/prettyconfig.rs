// Main entry point for prettyconfig TUI
// Contains run() function and event loop

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::prettyconfig::navigation::App;
use crate::prettyconfig::render::draw;
use crate::visuals::{colorcontrol, renderer};

// Run the prettyconfig TUI application
pub fn run() -> io::Result<()> {
    let config = crate::configloader::load_config();
    colorcontrol::init_colors(config.colors.clone());
    renderer::init_box_styles(config.box_style, config.border_line_style);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::from_config(&config);

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
            Event::Mouse(mouse) => {
                app.handle_mouse(mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }

        if app.should_exit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(msg) = &app.status_message {
        println!("{}", msg);
    }

    Ok(())
}
