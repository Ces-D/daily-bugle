use std::fmt::Display;

use anyhow::Result;
use clap::Parser;
use config::configuration::Config;
use third_party_api::{
    news::{self, request_response::Article},
    weather,
};
use web_scraper::time_out::{self, ArticleContent};

#[derive(Debug, Parser)]
pub struct GoodMorningArgs {
    #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
    model: Option<String>,
}

pub async fn handle_good_morning_command(args: GoodMorningArgs, config: Config) -> Result<()> {
    let timeout_events = time_out::scrape_things_to_do(time_out::ThingsToDoCycle::Today).await?;
    let weather_config = config
        .weather
        .expect("Weather config required for this command");
    let weather = weather::get_realtime_weather(weather::RealtimeWeatherApiUrl {
        api_key: weather_config.api_key,
        postal_code: weather_config.postal_code,
        ..Default::default()
    })
    .await?;
    let weather_summary = weather::summarize_weather(weather, "gpt-5-nano-2025-08-07").await?;
    let news_config = config.news.expect("News config required for this command");
    let top_news = news::top_headlines(news::TopHeadlinesUrl {
        api_key: news_config.api_key,
        sources: news_config.sources,
        ..Default::default()
    })
    .await?;
    let out = GoodMorningDisplay {
        weather_report: weather_summary,
        social: timeout_events.articles(),
        news: top_news.articles,
    };

    println!("{}", out);
    Ok(())
}

struct GoodMorningDisplay<'a> {
    pub weather_report: String,
    pub social: &'a Vec<ArticleContent>,
    pub news: Vec<Article>,
}

impl<'a> Display for GoodMorningDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::from("Good Morning");
        out.push_str(&format!("\n{}\n", self.weather_report));
        out.push_str(&format!(
            "\nNYC Events\n{:?}\n",
            serde_json::to_string_pretty(
                &self
                    .social
                    .iter()
                    .map(|v| v.title.clone().replace('\u{a0}', " ").replace('\n', ""))
                    .collect::<Vec<String>>()
            )
            .unwrap()
        ));
        out.push_str(&format!(
            "\nTop Headlines\n{:?}\n",
            serde_json::to_string_pretty(&self.news)
        ));
        write!(f, "{}", out)
    }
}
