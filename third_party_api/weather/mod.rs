pub mod request_response;

use crate::IntoUrl;
use anyhow::{Context, Result, bail};
use log::trace;
use reqwest::{
    StatusCode,
    header::{ACCEPT, ACCEPT_ENCODING},
};
use std::{io::Read, str::FromStr};

const WEATHER_API: &str = "https://api.tomorrow.io/v4/";

#[derive(Debug, Default)]
pub struct RealtimeWeatherApiUrl {
    pub api_key: String,
    pub postal_code: String,
    pub units: Option<String>,
}

impl IntoUrl for RealtimeWeatherApiUrl {
    fn into_url(self) -> url::Url {
        let mut url = url::Url::from_str(WEATHER_API).unwrap();
        url = url
            .join("weather/realtime")
            .expect("Failed to join weather url");
        url.query_pairs_mut()
            .append_pair("apikey", &self.api_key)
            .append_pair("location", &self.postal_code)
            .append_pair(
                "units",
                self.units
                    .unwrap_or_else(|| "imperial".to_string())
                    .as_str(),
            );
        trace!("Weather Realtime url: {:?}", url.to_string());
        url
    }
}

/// see - https://docs.tomorrow.io/reference/realtime-weather
pub async fn get_realtime_weather(
    url: RealtimeWeatherApiUrl,
) -> Result<request_response::RealtimeWeather> {
    let client = reqwest::Client::new();
    let res = client
        .get(url.into_url())
        .header(ACCEPT, "application/json")
        .header(ACCEPT_ENCODING, "deflate, gzip, br")
        .send()
        .await
        .with_context(|| "Failed to make get_todays_data request")?;
    if res.status() == StatusCode::OK {
        let bytes = res
            .bytes()
            .await
            .with_context(|| "Failed to decode response body")?;
        let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut decrypted = String::new();
        decoder.read_to_string(&mut decrypted)?;

        match serde_json::from_str::<request_response::RealtimeWeather>(&decrypted) {
            Ok(body) => Ok(body),
            Err(e) => {
                bail!("Failed to parse realtime weather response body: {}", e)
            }
        }
    } else {
        bail!(
            "Error requesting todays weather forecast: Status {}",
            res.status(),
        )
    }
}
