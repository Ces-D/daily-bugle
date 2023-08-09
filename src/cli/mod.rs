use clap::Command;

mod config;
mod weather_command;

pub async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::read_config();

    let cmd = Command::new("bugle")
        .bin_name("bugle")
        .subcommand_required(true)
        .subcommand(weather_command::create_command());

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("weather", matches)) => weather_command::handle_command(matches, config).await?,
        // Some(("news", matches)) => news_command_factory.handle(matches),
        _ => unreachable!("clap should have caught this"),
    };

    Ok(())
}
