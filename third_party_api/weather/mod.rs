pub mod request_response;

use crate::{
    IntoUrl, developer_message, make_chat_completion_request, request_url, system_message,
};
use anyhow::{Context, Result};
use async_openai::types::CreateChatCompletionRequestArgs;
use local_storage::key::StorageKey;
use log::trace;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, HeaderMap, HeaderValue};
use std::str::FromStr;

const REALTIME_WEATHER_API_STORAGE_CONSTANT: &str = "tomorrowIO_realtime_weather";
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
    match local_storage::find_stored_item(REALTIME_WEATHER_API_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let mut headers = HeaderMap::new();
            headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
            headers.insert(
                ACCEPT_ENCODING,
                HeaderValue::from_str("deflate, gzip, br").unwrap(),
            );
            let res = request_url::<request_response::RealtimeWeather>(
                url.into_url().as_str(),
                Some(headers),
            )
            .await?;
            let storage_key = StorageKey::new(REALTIME_WEATHER_API_STORAGE_CONSTANT, None, Some(2));
            local_storage::write_item_to_storage(storage_key, &res).await;
            Ok(res)
        }
    }
}

const SUMMARIZE_WEATHER_PROMPT:&str="You are a Weather Report Generator. You will receive a JSON object containing: data.time (an ISO timestamp), data.values (raw weather measurements), and location (latitude, longitude, and a human-readable location name). Your job is to transform this input into a clear, concise, human-friendly current weather report.

Begin with a one-sentence summary describing the overall weather (temperature and general conditions). Present the report in short sections with the following information when available: Temperature (actual and apparent in °C), Sky conditions (interpret cloudCover 0–100% as clear, mostly clear, partly cloudy, or overcast), Weather code (convert into plain-language description, e.g., 1001 = overcast), Wind (speed in m/s and km/h, plus direction as a compass direction), Precipitation (summarize rain, snow, sleet, freezing rain; if all intensities are zero, say “no precipitation”; include precipitation probability), Visibility in kilometers, Humidity percentage, Pressure in millibars with a note if it is generally low, normal, or high, and UV Index with health concern level.

If any value is missing, omit it. Tone must be neutral, clear, and readable, avoiding jargon. Output text only.

Cloud cover interpretation rules: 0–10% = clear; 10–40% = mostly clear; 40–70% = partly cloudy; 70–100% = overcast. Precipitation rules: if all intensities are zero, state no precipitation; if probability is 0, state no expected precipitation.

Output format: “Current Weather in <location_name> (as of <local_time>):” followed by the summary sentence and sections for Temperature, Sky, Wind, Precipitation, Visibility, Humidity, Pressure, and UV Index.";

pub async fn summarize_weather(
    response: request_response::RealtimeWeather,
    model: &str,
) -> Result<String> {
    let request = CreateChatCompletionRequestArgs::default()
        .messages(vec![
            system_message(SUMMARIZE_WEATHER_PROMPT),
            developer_message(serde_json::to_string(&response).unwrap()),
        ])
        .model(model)
        .build()
        .with_context(|| "Failed to create summarize weather chat completion request")?;
    let summarization = make_chat_completion_request(request).await?;
    Ok(summarization)
}
