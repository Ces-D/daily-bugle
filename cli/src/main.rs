mod good_morning_command;
mod logger;
mod reminder_command;
mod social_command;
mod tech_command;
mod tool_command;

use anyhow::Ok;
use clap::Parser;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "Commands related to technical subjects")]
    Technical(tech_command::TechArgs),

    #[clap(about = "Commands related to socialization")]
    Social(social_command::SocialArgs),

    #[clap(about = "Commands of a variety")]
    Tool(tool_command::ToolArgs),

    #[clap(about = "Set or get countdowns ")]
    Reminder(reminder_command::ReminderArgs),

    #[clap(about = "Ge the morning briefing")]
    GoodMorning(good_morning_command::GoodMorningArgs),

    TestNode,
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
    let bugle_config = config::read_config_file()?;

    match app.command {
        Command::Social(args) => social_command::handle_social_command(args, bugle_config).await,

        Command::Technical(args) => tech_command::handle_tech_command(args, bugle_config).await,

        Command::Reminder(args) => reminder_command::handle_reminder_command(args).await,

        Command::GoodMorning(args) => {
            good_morning_command::handle_good_morning_command(args, bugle_config).await
        }

        Command::Tool(tool_args) => {
            tool_command::handle_tool_command(
                tool_args,
                bugle_config
                    .career
                    .expect("Career config must be populated"),
            )
            .await
        }

        Command::TestNode => {
            let o = std::process::Command::new("node")
                .arg("google/dist/init.js")
                .arg("test")
                .output()
                .expect("Failed to execute node script");
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                println!("Node response: {}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("Node.js script failed with error: {}", stderr.trim());
            }
            Ok(())
        }
    }
}
