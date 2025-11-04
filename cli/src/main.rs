mod app;
mod logger;
mod reminder_command;
mod social_command;
mod tech_command;

use anyhow::{Context, Ok, bail};
use clap::Parser;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logging();
    let app = app::App::parse();
    let bugle_config = config::read_config_file()?;

    match app.command {
        app::Command::Weather {
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
        app::Command::Social(args) => social_command::handle_social_command(args).await,

        app::Command::Technical(args) => tech_command::handle_tech_command(args).await,

        app::Command::Reminder(args) => reminder_command::handle_reminder_command(args).await,

        app::Command::TestNode => {
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
