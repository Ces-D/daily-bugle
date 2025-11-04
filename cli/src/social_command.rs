use clap::{Parser, Subcommand, ValueEnum};

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
    Timeout { period: TimeOutTimePeriod },
}

#[derive(Debug, Parser)]
pub struct SocialArgs {
    #[clap(subcommand)]
    pub command: SocialCommand,
}

pub async fn handle_social_command(args: SocialArgs) -> anyhow::Result<()> {
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
            serde_json::to_writer_pretty(std::io::stdout(), &out)?;
            Ok(())
        }
    }
}
