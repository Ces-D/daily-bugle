pub mod response;

use anyhow::{Context, bail};
use reqwest::{
    StatusCode,
    header::{ACCEPT, ACCEPT_ENCODING},
};
use std::str::FromStr;

const WEATHER_API: &str = "https://api.tomorrow.io/v4/";

/// see - https://docs.tomorrow.io/reference/realtime-weather
pub async fn get_realtime_weather(
    api_key: &str,
    postal_code: &str,
) -> anyhow::Result<response::RealtimeWeather> {
    let url = url::Url::from_str(WEATHER_API).unwrap();
    url.join("weather/realtime")?
        .query_pairs_mut()
        .append_pair("apikey", api_key)
        .append_pair("location", postal_code)
        .append_pair("units", "imperial");
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header(ACCEPT_ENCODING, "deflate, gzip, br")
        .send()
        .await
        .with_context(|| "Failed to make get_todays_data request")?;
    if res.status() == StatusCode::OK {
        match res.json::<response::RealtimeWeather>().await {
            Ok(body) => Ok(body),
            Err(e) => bail!("Failed to parse request body: {}", e),
        }
    } else {
        bail!(
            "Error requesting todays weather forecast: Status {}",
            res.status(),
        )
    }
}
