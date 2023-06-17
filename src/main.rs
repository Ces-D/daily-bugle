use clap::Command;
mod news_command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let news_command_factory = news_command::NewsCommandFactory::new();
    let news_command = news_command_factory.generate();

    let cmd = Command::new("bugle")
        .bin_name("bugle")
        .subcommand_required(true)
        .subcommand(news_command);

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("news", matches)) => news_command_factory.handle(matches),
        _ => unreachable!("clap should have caught this"),
    };

    Ok(())
}
