use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RealtimeWeatherDataValues {
    pub cloud_base: u32,
    pub cloud_ceiling: u32,
    pub cloud_cover: u32,
    pub dew_point: u32,
    pub freezing_rain_intensity: u32,
    pub humidity: u32,
    pub precipitation_probability: u32,
    pub pressure_surface_level: u32,
    pub rain_intensity: u32,
    pub sleet_intensity: u32,
    pub snow_intensity: u32,
    pub temperature: u32,
    pub temperature_apparent: u32,
    pub uv_health_concern: u32,
    pub uv_index: u32,
    pub visibility: u32,
    pub weather_code: u32,
    pub wind_direction: u32,
    pub wind_gust: u32,
    pub wind_speed: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeatherData {
    pub time: String,
    pub values: RealtimeWeatherDataValues,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeatherLocation {
    pub lat: u32,
    pub lon: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeather {
    pub data: RealtimeWeatherData,
    pub location: RealtimeWeatherLocation,
}
