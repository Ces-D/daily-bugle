use clap::{Parser, Subcommand, ValueEnum};
use log::{info, trace};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TimeOutTimePeriod {
    Today,
    Week,
    Weekend,
    Month,
}

#[derive(Debug, Subcommand)]
pub enum SocialCommand {
    #[clap(about = "Get a list of things to do in nyc")]
    Timeout {
        period: TimeOutTimePeriod,
    },
    Find {
        query: String,
    },
}

#[derive(Debug, Parser)]
pub struct SocialArgs {
    #[clap(subcommand)]
    pub command: SocialCommand,
}

pub async fn handle_social_command(
    args: SocialArgs,
    config: config::configuration::Config,
) -> anyhow::Result<()> {
    trace!("Starting handle_social_command");
    match args.command {
        SocialCommand::Timeout { period } => {
            let out = match period {
                TimeOutTimePeriod::Today => {
                    web_scraper::time_out::scrape_things_to_do(
                        web_scraper::time_out::ThingsToDoCycle::Today,
                    )
                    .await?
                }
                TimeOutTimePeriod::Week => {
                    web_scraper::time_out::scrape_things_to_do(
                        web_scraper::time_out::ThingsToDoCycle::Week,
                    )
                    .await?
                }
                TimeOutTimePeriod::Weekend => {
                    web_scraper::time_out::scrape_things_to_do(
                        web_scraper::time_out::ThingsToDoCycle::Weekend,
                    )
                    .await?
                }
                TimeOutTimePeriod::Month => {
                    web_scraper::time_out::scrape_things_to_do(
                        web_scraper::time_out::ThingsToDoCycle::Month,
                    )
                    .await?
                }
            };
            info!("Success scraping: {} results", out.len());
            serde_json::to_writer_pretty(std::io::stdout(), &out)?;
            Ok(())
        }
        SocialCommand::Find { query } => {
            todo!()
        }
    }
}

struct QueryMetadata {
    period: chrono::Duration,
    interests: Vec<String>,
}
