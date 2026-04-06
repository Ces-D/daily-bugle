use clap::Parser;
use config::read_config_file;

mod commands;
mod logger;
mod tools;

// TODO: redo all of this to be tools that the agent core calls. It should only work with these
// tools and what the llm knows
// Add a konan tool

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "Commands related to technical subjects")]
    Technical(commands::tech_command::TechArgs),
    #[clap(about = "Commands related to cryptography")]
    Fortress(commands::fortress_command::FortressArgs),
    #[clap(about = "Commands related to almanac")]
    Almanac(commands::almanac_command::AlmanacArgs),
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(
        short,
        long,
        help = "The profile known_as associated with this call",
        global = true
    )]
    pub profile: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logging();
    let app = App::parse();
    let config = read_config_file()?;
    let profile = if let Some(p) = app.profile {
        config.profile.iter().find(|v| v.known_as == p)
    } else {
        None
    };

    match app.command {
        Command::Technical(args) => commands::tech_command::handle_tech_command(args).await,
        Command::Fortress(args) => commands::fortress_command::handle_fortress_command(args).await,
        Command::Almanac(args) => {
            commands::almanac_command::handle_almanac_command(args, profile).await
        }
    }
}
