mod logger;

use anyhow::{Context, bail};
use clap::{Parser, ValueEnum};
use log::trace;
use serde_json::json;
use time_out::scrape::{ThingsToDoCycle, scrape_things_to_do};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TimePeriod {
    Today,
    Week,
    Weekend,
    Month,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Weather { postal_code: String, complete: bool },
    TimeOut { events: TimePeriod },
    TestNode,
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
struct App {
    #[clap(subcommand)]
    command: Command,
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
        Command::TimeOut { events } => {
            let things_to_do = match events {
                TimePeriod::Today => {
                    scrape_things_to_do(
                        ThingsToDoCycle::Today,
                        config::local_storage_dir_location()?,
                    )
                    .await?
                }
                TimePeriod::Week => {
                    scrape_things_to_do(
                        ThingsToDoCycle::Week,
                        config::local_storage_dir_location()?,
                    )
                    .await?
                }
                TimePeriod::Weekend => {
                    scrape_things_to_do(
                        ThingsToDoCycle::Weekend,
                        config::local_storage_dir_location()?,
                    )
                    .await?
                }
                TimePeriod::Month => {
                    scrape_things_to_do(
                        ThingsToDoCycle::Month,
                        config::local_storage_dir_location()?,
                    )
                    .await?
                }
            };
            trace!("Timeout request complete");
            let writer = std::io::stdout();
            serde_json::to_writer_pretty(writer, &things_to_do)?;
            Ok(())
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
