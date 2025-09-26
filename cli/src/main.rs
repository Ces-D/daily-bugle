use anyhow::{bail, Context};
use clap::Parser;
use serde_json::json;

#[derive(Debug, clap::Subcommand)]
enum Command {
    Weather { postal_code: String, complete: bool },
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
                    println!("{}", out)
                } else {
                    let out = json!({
                        "location": &res.location.name,
                        "time": &res.data.time,
                        "temperature": &res.data.values.temperature,
                        "feels_like": &res.data.values.temperature_apparent
                    });
                    println!("{}", out)
                }
                Ok(())
            } else {
                bail!("Config.weather must be populated")
            }
        }
    }
}
