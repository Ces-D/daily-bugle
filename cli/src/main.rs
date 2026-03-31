mod fortress_command;
mod good_morning_command;
mod logger;
mod social_command;
mod tech_command;

use clap::Parser;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "Commands related to technical subjects")]
    Technical(tech_command::TechArgs),
    #[clap(about = "Commands related to socialization")]
    Social(social_command::SocialArgs),
    #[clap(about = "Ge the morning briefing")]
    GoodMorning(good_morning_command::GoodMorningArgs),
    #[clap(about = "Commands relateed to cryptography")]
    Fortress(fortress_command::FortressArgs),
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logging();

    let app = App::parse();

    match app.command {
        Command::Social(args) => social_command::handle_social_command(args).await,
        Command::Technical(args) => tech_command::handle_tech_command(args).await,
        Command::GoodMorning(args) => good_morning_command::handle_good_morning_command(args).await,
        Command::Fortress(args) => fortress_command::handle_fortress_command(args).await,
    }
}
