use std::error::Error;

mod api_client;
mod cli;
mod ui;

// see - https://github.com/16arpi/meteo-tui
// TODO: consider ratatui
fn main() -> Result<(), Box<dyn Error>> {
    cli::run_app();
    Ok(())
}
