mod logger;
mod reminder_command;
mod social_command;
mod tech_command;

use anyhow::{Context, Ok, bail};
use clap::Parser;
use serde_json::json;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "See the weather in your area")]
    Weather {
        postal_code: String,
        complete: bool,
    },

    #[clap(about = "Get a random article from the list of technical article sources")]
    Technical(tech_command::TechArgs),

    Social(social_command::SocialArgs),

    Reminder(reminder_command::ReminderArgs),
    #[clap(about = "Set or get countdowns ")]
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
        Command::Weather {
            postal_code,
            complete,
        } => {
            if let Some(weather_config) = bugle_config.weather {
                let res =
                    weather::get_realtime_weather(&weather_config.api_key, &postal_code).await?;
                if complete {
                    let out = serde_json::to_string_pretty(&res)
                        .with_context(|| "Failed to convert weather response")?;
                    serde_json::to_writer_pretty(std::io::stdout(), &out)?;
                } else {
                    let out = json!({
                        "location": &res.location.name,
                        "time": &res.data.time,
                        "temperature": &res.data.values.temperature,
                        "feels_like": &res.data.values.temperature_apparent
                    });
                    serde_json::to_writer_pretty(std::io::stdout(), &out)?;
                }
                Ok(())
            } else {
                bail!("Config.weather must be populated")
            }
        }

        Command::Social(args) => social_command::handle_social_command(args, bugle_config).await,

        Command::Technical(args) => tech_command::handle_tech_command(args).await,

        Command::Reminder(args) => reminder_command::handle_reminder_command(args).await,

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
