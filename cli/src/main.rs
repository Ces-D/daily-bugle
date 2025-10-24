mod command;
mod logger;

use anyhow::{Context, bail};
use clap::Parser;
use command::Command;
use serde_json::json;
use web_scraper::ScrapedEngineeringItems;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logging();
    let app = command::App::parse();
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

        Command::NYCEvent { period } => todo!(),

        Command::TechnicalArticle { sources } => {
            let mut entries: ScrapedEngineeringItems = vec![];
            for source in sources {
                match source {
                    command::TechnicalArticleSource::Aws => {
                        let aws = web_scraper::aws::scrape_aws_engineering_sitemap().await?;
                        entries.extend(aws);
                    }
                    command::TechnicalArticleSource::Figma => {
                        let figma = web_scraper::figma::scrape_figma_engineering_blog().await?;
                        entries.extend(figma);
                    }
                    command::TechnicalArticleSource::Google => {
                        let google =
                            web_scraper::google::scrape_google_developer_blogs_sitemap().await?;
                        entries.extend(google);
                    }
                    command::TechnicalArticleSource::HackernewsNews => {
                        let hackernews_news =
                            web_scraper::hackernews::scrape_hackernews_news(None).await?;
                        entries.extend(hackernews_news);
                    }
                    command::TechnicalArticleSource::HackernewsJobs => {
                        let hackernews_jobs =
                            web_scraper::hackernews::scrape_hackernews_jobs(None).await?;
                        entries.extend(hackernews_jobs);
                    }
                    command::TechnicalArticleSource::ArminRonacher => {
                        let lucumr = web_scraper::lucumr::scrape_lucumr_atom_feed().await?;
                        entries.extend(lucumr);
                    }
                    command::TechnicalArticleSource::Mdn => {
                        let mdn = web_scraper::mdn::scrape_mdn_sitemap().await?;
                        entries.extend(mdn);
                    }
                    command::TechnicalArticleSource::Notion => {
                        let notion = web_scraper::notion::scrape_notion_blog_sitemap().await?;
                        entries.extend(notion);
                    }
                    command::TechnicalArticleSource::Openai => {
                        let openai = web_scraper::openai::scrape_openai_sitemap().await?;
                        entries.extend(openai);
                    }
                    command::TechnicalArticleSource::Uber => {
                        let uber = web_scraper::uber::scrape_uber_engineering_blog().await?;
                        entries.extend(uber);
                    }
                }
            }
            let random_index = rand::random_range(..entries.len());
            let random_entry = entries.get(random_index).unwrap();
            let out = serde_json::to_string_pretty(&random_entry)
                .with_context(|| "Failed to convert weather response")?;
            println!("{}", out);
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
