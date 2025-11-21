use clap::Parser;
use config::configuration::Config;
use third_party_api::{news, weather};
use web_scraper::time_out;

#[derive(Debug, Parser)]
pub struct GoodMorningArgs {
    #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
    model: Option<String>,
}

pub async fn handle_good_morning_command(
    args: GoodMorningArgs,
    config: Config,
) -> anyhow::Result<()> {
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
    let news_config = config.news.expect("News config required for this command");
    let top_news = news::top_headlines(news::TopHeadlinesUrl {
        api_key: news_config.api_key,
        sources: news_config.sources,
        ..Default::default()
    })
    .await?;
    let out = serde_json::json!({
            "weather": {
                "temperature": weather.data.values.temperature,
                "rain_intensity": weather.data.values.rain_intensity,
            },
            "social": timeout_events.articles(),
            "news": top_news.sources,
    });

    serde_json::to_writer_pretty(&std::io::stdout(), &out)?;
    Ok(())
}
