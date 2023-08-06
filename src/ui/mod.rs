use std::io::{Result, Stdout};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

pub fn create_terminal_in_raw_mode() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().expect("unable to enable raw mode");

    let mut buffer = std::io::stdout();
    crossterm::execute!(buffer, EnterAlternateScreen).expect("unable to enter alternate screen");
    let backend = CrosstermBackend::new(buffer);
    Terminal::new(backend)
}

pub fn cleanup_terminal_in_raw_mode(buffer: &mut std::io::Stdout) -> Result<()> {
    disable_raw_mode().expect("unable to disable raw mode");
    crossterm::execute!(buffer, LeaveAlternateScreen).expect("unable to leave alternate screen");
    Ok(())
}

// TODO: test these two functions but first connect their initialization to a clap command
