use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RealtimeWeatherDataValues {
    pub cloud_base: Option<f32>,
    pub cloud_ceiling: Option<f32>,
    pub cloud_cover: Option<u32>,
    pub dew_point: Option<f32>,
    pub freezing_rain_intensity: Option<u32>,
    pub humidity: Option<u32>,
    pub precipitation_probability: Option<u32>,
    pub pressure_surface_level: Option<f32>,
    pub rain_intensity: Option<u32>,
    pub sleet_intensity: Option<u32>,
    pub snow_intensity: Option<u32>,
    pub temperature: Option<f32>,
    pub temperature_apparent: Option<f32>,
    pub uv_health_concern: Option<u32>,
    pub uv_index: Option<u32>,
    pub visibility: Option<f32>,
    pub weather_code: Option<u32>,
    pub wind_direction: Option<u32>,
    pub wind_gust: Option<f32>,
    pub wind_speed: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeatherData {
    pub time: String,
    pub values: RealtimeWeatherDataValues,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeatherLocation {
    pub lat: Option<f32>,
    pub lon: Option<f32>,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealtimeWeather {
    pub data: RealtimeWeatherData,
    pub location: RealtimeWeatherLocation,
}
