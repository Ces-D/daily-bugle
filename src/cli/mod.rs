use clap::Command;

mod weather_command;

pub fn run_app() {
    let cmd = Command::new("bugle")
        .bin_name("bugle")
        .subcommand_required(true)
        .subcommand(weather_command::create_command());

    let matches = cmd.get_matches();
    match matches.subcommand() {
        // Some(("news", matches)) => news_command_factory.handle(matches),
        _ => unreachable!("clap should have caught this"),
    };
}
