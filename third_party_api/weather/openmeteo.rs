use anyhow::{Context, Result, bail};
use chrono::{DateTime, TimeZone, Utc};
use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

const TIMEFORMAT: &str = "unixtime";
const WIND_SPEED_UNIT: &str = "mph";
const TEMPERATURE_UNIT: &str = "fahrenheit";
const PRECIPITATION_UNIT: &str = "inch";
const FORECAST_DAYS: u8 = 7;
const OPEN_METEO_FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

// ─── Temperature ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Temperature {
    pub degrees: f64,
}

impl Temperature {
    pub fn as_fahrenheit(&self) -> String {
        format!("{:.1}°F", self.degrees)
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_fahrenheit())
    }
}

// ─── WMO Weather Code ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct WmoWeatherCode {
    pub code: u16,
}

impl WmoWeatherCode {
    pub fn description(&self) -> &'static str {
        match self.code {
            0 => {
                "Cloud development not observed or not observable (change in sky state over past hour)"
            }
            1 => "Clouds generally dissolving or becoming less developed",
            2 => "State of sky on the whole unchanged",
            3 => "Clouds generally forming or developing",
            4 => "Visibility reduced by smoke (e.g., fires, industrial smoke, volcanic ash)",
            5 => "Haze",
            6 => "Widespread dust in suspension (not raised by wind at station)",
            7 => "Dust or sand raised by wind at station (no whirls/storms seen)",
            8 => "Well-developed dust or sand whirls seen (no storm)",
            9 => "Duststorm or sandstorm within sight or at station recently",
            10 => "Mist",
            11 => "Patches of shallow fog or ice fog at station (low depth)",
            12 => "More or less continuous shallow fog or ice fog at station",
            13 => "Lightning visible, no thunder heard",
            14 => "Precipitation visible, not reaching the ground/sea surface",
            15 => "Precipitation visible, reaching ground/sea surface but distant (> 5 km)",
            16 => "Precipitation visible, near but not at station",
            17 => "Thunderstorm, no precipitation at observation time",
            18 => "Squalls at or within sight of station",
            19 => "Funnel cloud(s) (tornado/waterspout)",
            20 => "Drizzle (not freezing) or snow grains, not falling as showers",
            21 => "Rain (not freezing), not falling as showers",
            22 => "Snow, not falling as showers",
            23 => "Rain and snow or ice pellets, not in showers",
            24 => "Freezing drizzle or freezing rain, not in showers",
            25 => "Shower(s) of rain, within past hour",
            26 => "Shower(s) of snow or rain and snow, within past hour",
            27 => "Shower(s) of hail, rain and hail, within past hour",
            28 => "Fog or ice fog within past hour",
            29 => "Thunderstorm (with or without precipitation) within past hour",
            30 => "Slight/moderate duststorm or sandstorm — has decreased",
            31 => "Slight/moderate duststorm or sandstorm — steady",
            32 => "Slight/moderate duststorm or sandstorm — begun or increased",
            33 => "Severe duststorm or sandstorm — has decreased",
            34 => "Severe duststorm or sandstorm — steady",
            35 => "Severe duststorm or sandstorm — begun or increased",
            36 => "Slight/moderate drifting snow, generally low",
            37 => "Heavy drifting snow, generally low",
            38 => "Slight/moderate blowing snow, generally high",
            39 => "Heavy blowing snow, generally high",
            40 => "Fog or ice fog at a distance at observation (not recent at station)",
            41 => "Fog or ice fog in patches",
            42 => "Fog or ice fog, sky visible — has become thinner",
            43 => "Fog or ice fog, sky invisible",
            44 => "Fog or ice fog, sky visible — no change",
            45 => "Fog or ice fog, sky invisible — no change",
            46 => "Fog or ice fog, sky visible — has begun or thickened",
            47 => "Fog or ice fog, sky invisible — has begun or thickened",
            48 => "Fog depositing rime, sky visible",
            49 => "Fog depositing rime, sky invisible",
            50 => "Drizzle, not freezing, intermittent — slight now",
            51 => "Drizzle, not freezing, continuous",
            52 => "Drizzle, not freezing, intermittent — moderate now",
            53 => "Drizzle, not freezing, continuous",
            54 => "Drizzle, not freezing, intermittent — heavy now",
            55 => "Drizzle, not freezing, continuous",
            56 => "Drizzle, freezing — slight",
            57 => "Drizzle, freezing — moderate or heavy",
            58 => "Drizzle and rain — slight",
            59 => "Drizzle and rain — moderate or heavy",
            60 => "Rain, not freezing, intermittent — slight now",
            61 => "Rain, not freezing, continuous",
            62 => "Rain, not freezing, intermittent — moderate now",
            63 => "Rain, not freezing, continuous",
            64 => "Rain, not freezing, intermittent — heavy now",
            65 => "Rain, not freezing, continuous",
            66 => "Rain, freezing — slight",
            67 => "Rain, freezing — moderate or heavy",
            68 => "Rain or drizzle and snow — slight",
            69 => "Rain or drizzle and snow — moderate or heavy",
            70 => "Intermittent snowflakes — slight now",
            71 => "Continuous snowflakes",
            72 => "Intermittent snowflakes — moderate now",
            73 => "Continuous snowflakes",
            74 => "Intermittent snowflakes — heavy now",
            75 => "Continuous snowflakes — heavy",
            76 => "Diamond dust (with or without fog)",
            77 => "Snow grains (with or without fog)",
            78 => "Isolated star-like snow crystals",
            79 => "Ice pellets",
            80 => "Rain showers — slight",
            81 => "Rain showers — moderate or heavy",
            82 => "Rain showers — violent",
            83 => "Showers of rain & snow mixed — slight",
            84 => "Showers of rain & snow mixed — moderate or heavy",
            85 => "Snow showers — slight",
            86 => "Snow showers — moderate or heavy",
            87 => "Showers of snow pellets or small hail — slight",
            88 => "Showers of snow pellets or small hail — moderate or heavy",
            89 => "Hail showers (no thunder) — slight",
            90 => "Hail showers (no thunder) — moderate or heavy",
            91 => "Slight rain now; thunderstorm occurred in past hour",
            92 => "Moderate/heavy rain now",
            93 => "Slight snow or mixed precipitation now",
            94 => "Moderate/heavy snow or mixed precipitation now",
            95 => "Thunderstorm — slight/moderate with rain/snow now",
            96 => "Thunderstorm — slight/moderate with hail now",
            97 => "Thunderstorm — heavy, no hail now",
            98 => "Thunderstorm combined with dust/sandstorm now",
            99 => "Thunderstorm — heavy with hail now",
            _ => "",
        }
    }
}

// ─── Supported field types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DailyField {
    WeatherCode,
    Sunrise,
    Sunset,
    TemperatureMin,
    TemperatureMax,
}

impl DailyField {
    fn as_api_str(&self) -> &'static str {
        match self {
            Self::WeatherCode => "weather_code",
            Self::Sunrise => "sunrise",
            Self::Sunset => "sunset",
            Self::TemperatureMin => "temperature_2m_min",
            Self::TemperatureMax => "temperature_2m_max",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HourlyField {
    Temperature,
    WeatherCode,
}

impl HourlyField {
    fn as_api_str(&self) -> &'static str {
        match self {
            Self::Temperature => "temperature_2m",
            Self::WeatherCode => "weather_code",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CurrentField {
    Temperature,
    WeatherCode,
    ApparentTemperature,
}

impl CurrentField {
    fn as_api_str(&self) -> &'static str {
        match self {
            Self::Temperature => "temperature_2m",
            Self::WeatherCode => "weather_code",
            Self::ApparentTemperature => "apparent_temperature",
        }
    }
}

// ─── Open-Meteo JSON Response Types ─────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentData {
    pub time: i64,
    #[serde(default)]
    pub temperature_2m: Option<f32>,
    #[serde(default)]
    pub weather_code: Option<u16>,
    #[serde(default)]
    pub apparent_temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DailyData {
    pub time: Vec<i64>,
    #[serde(default)]
    pub weather_code: Option<Vec<u16>>,
    #[serde(default)]
    pub sunrise: Option<Vec<i64>>,
    #[serde(default)]
    pub sunset: Option<Vec<i64>>,
    #[serde(default)]
    pub temperature_2m_min: Option<Vec<f32>>,
    #[serde(default)]
    pub temperature_2m_max: Option<Vec<f32>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HourlyData {
    pub time: Vec<i64>,
    #[serde(default)]
    pub temperature_2m: Option<Vec<f32>>,
    #[serde(default)]
    pub weather_code: Option<Vec<u16>>,
}

#[derive(Debug, Deserialize)]
pub struct WeatherApiResponse {
    pub latitude: f32,
    pub longitude: f32,
    #[serde(default)]
    pub elevation: Option<f32>,
    #[serde(default)]
    pub utc_offset_seconds: i32,
    #[serde(default)]
    pub current: Option<CurrentData>,
    #[serde(default)]
    pub daily: Option<DailyData>,
    #[serde(default)]
    pub hourly: Option<HourlyData>,
}

// ─── WeatherForecastBuilder ─────────────────────────────────────────────────

pub struct WeatherForecastBuilder {
    latitude: f64,
    longitude: f64,
    daily: Option<BTreeSet<DailyField>>,
    hourly: Option<BTreeSet<HourlyField>>,
    current: Option<BTreeSet<CurrentField>>,
}

impl WeatherForecastBuilder {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            daily: None,
            hourly: None,
            current: None,
        }
    }

    pub fn daily(mut self, fields: impl IntoIterator<Item = DailyField>) -> Self {
        self.daily = Some(fields.into_iter().collect());
        self
    }

    pub fn hourly(mut self, fields: impl IntoIterator<Item = HourlyField>) -> Self {
        self.hourly = Some(fields.into_iter().collect());
        self
    }

    pub fn current(mut self, fields: impl IntoIterator<Item = CurrentField>) -> Self {
        self.current = Some(fields.into_iter().collect());
        self
    }

    pub async fn send(&self) -> Result<WeatherForecast> {
        let mut params: Vec<(&str, String)> = vec![
            ("latitude", self.latitude.to_string()),
            ("longitude", self.longitude.to_string()),
            ("timeformat", TIMEFORMAT.to_string()),
            ("forecast_days", FORECAST_DAYS.to_string()),
            ("wind_speed_unit", WIND_SPEED_UNIT.to_string()),
            ("temperature_unit", TEMPERATURE_UNIT.to_string()),
            ("precipitation_unit", PRECIPITATION_UNIT.to_string()),
        ];

        if let Some(ref daily) = self.daily {
            if !daily.is_empty() {
                let val: Vec<&str> = daily.iter().map(DailyField::as_api_str).collect();
                params.push(("daily", val.join(",")));
            }
        }
        if let Some(ref hourly) = self.hourly {
            if !hourly.is_empty() {
                let val: Vec<&str> = hourly.iter().map(HourlyField::as_api_str).collect();
                params.push(("hourly", val.join(",")));
            }
        }
        if let Some(ref current) = self.current {
            let val: Vec<&str> = current.iter().map(CurrentField::as_api_str).collect();
            params.push(("current", val.join(",")));
        }

        let url = reqwest::Url::parse_with_params(OPEN_METEO_FORECAST_URL, &params)
            .with_context(|| "Failed to build Open-Meteo URL")?;

        trace!("Open-Meteo request URL: {url}");

        let response = reqwest::get(url.clone())
            .await
            .with_context(|| "Open-Meteo request failed")?;

        if !response.status().is_success() {
            bail!(
                "Open-Meteo returned status {} for {}",
                response.status(),
                url
            );
        }

        let data: WeatherApiResponse = response
            .json()
            .await
            .with_context(|| "Failed to deserialize Open-Meteo response")?;

        trace!("Open-Meteo response received");

        Ok(WeatherForecast {
            utc_offset_seconds: data.utc_offset_seconds as i64,
            current: data.current,
            daily: data.daily,
            hourly: data.hourly,
        })
    }
}

// ─── WeatherForecast ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct WeatherForecast {
    pub utc_offset_seconds: i64,
    pub current: Option<CurrentData>,
    pub daily: Option<DailyData>,
    pub hourly: Option<HourlyData>,
}

impl WeatherForecast {
    fn offset_to_datetime(&self, unix_seconds: i64) -> DateTime<Utc> {
        trace!(
            "offset_to_datetime: unix_seconds={}, utc_offset_seconds={}, adjusted={}",
            unix_seconds,
            self.utc_offset_seconds,
            unix_seconds + self.utc_offset_seconds
        );
        let dt = Utc
            .timestamp_opt(unix_seconds + self.utc_offset_seconds, 0)
            .single()
            .unwrap_or_default();
        trace!("offset_to_datetime: result={}", dt);
        dt
    }

    pub fn current_time(&self) -> Result<DateTime<Utc>> {
        let current = self.current.as_ref().context("No current data available")?;
        let dt = self.offset_to_datetime(current.time);
        trace!("currentTime: {}", dt);
        Ok(dt)
    }

    pub fn current_temperature(&self) -> Result<Temperature> {
        let current = self.current.as_ref().context("No current data available")?;
        let degrees = current
            .temperature_2m
            .context("No temperature in current data")? as f64;
        let temp = Temperature { degrees };
        trace!("currentTemperature: {temp}");
        Ok(temp)
    }

    pub fn current_apparent_temperature(&self) -> Result<Temperature> {
        let current = self.current.as_ref().context("No current data available")?;
        let degrees = current
            .apparent_temperature
            .context("No apparent temperature in current data")? as f64;
        let temp = Temperature { degrees };
        trace!("currentTemperature: {temp}");
        Ok(temp)
    }

    pub fn current_weather_code(&self) -> Result<WmoWeatherCode> {
        let current = self.current.as_ref().context("No current data available")?;
        let code = current
            .weather_code
            .context("No weather_code in current data")?;
        let wmo = WmoWeatherCode { code };
        trace!("currentWeatherCode: {} - {}", wmo.code, wmo.description());
        Ok(wmo)
    }

    pub fn hourly_weather_codes(&self) -> Result<Vec<(DateTime<Utc>, WmoWeatherCode)>> {
        let hourly = self.hourly.as_ref().context("No hourly data available")?;
        let codes = hourly
            .weather_code
            .as_ref()
            .context("No weather_code in hourly data")?;
        if hourly.time.len() != codes.len() {
            bail!(
                "Hourly time count ({}) does not match weather_code count ({})",
                hourly.time.len(),
                codes.len()
            );
        }
        let result: Vec<_> = hourly
            .time
            .iter()
            .zip(codes.iter())
            .map(|(&t, &c)| (self.offset_to_datetime(t), WmoWeatherCode { code: c }))
            .collect();
        trace!("hourlyWeatherCodes: {} entries", result.len());
        Ok(result)
    }

    pub fn hourly_temperatures(&self) -> Result<Vec<Temperature>> {
        let hourly = self.hourly.as_ref().context("No hourly data available")?;
        let temps = hourly
            .temperature_2m
            .as_ref()
            .context("No temperature in hourly data")?;
        let result: Vec<_> = temps
            .iter()
            .map(|&d| Temperature { degrees: d as f64 })
            .collect();
        trace!("hourlyTemperatures: {} entries", result.len());
        Ok(result)
    }

    pub fn daily_weather_codes(&self) -> Result<Vec<(DateTime<Utc>, WmoWeatherCode)>> {
        let daily = self.daily.as_ref().context("No daily data available")?;
        let codes = daily
            .weather_code
            .as_ref()
            .context("No weather_code in daily data")?;
        if daily.time.len() != codes.len() {
            bail!(
                "Daily time count ({}) does not match weather_code count ({})",
                daily.time.len(),
                codes.len()
            );
        }
        let result: Vec<_> = daily
            .time
            .iter()
            .zip(codes.iter())
            .map(|(&t, &c)| (self.offset_to_datetime(t), WmoWeatherCode { code: c }))
            .collect();
        trace!("dailyWeatherCodes: {} entries", result.len());
        Ok(result)
    }

    pub fn daily_sunrise(&self) -> Result<Vec<DateTime<Utc>>> {
        let daily = self.daily.as_ref().context("No daily data available")?;
        let sunrises = daily.sunrise.as_ref().context("No sunrise in daily data")?;
        let result: Vec<_> = sunrises
            .iter()
            .map(|&s| self.offset_to_datetime(s))
            .collect();
        trace!("dailySunrise: {:?}", result);
        Ok(result)
    }

    pub fn daily_sunset(&self) -> Result<Vec<DateTime<Utc>>> {
        let daily = self.daily.as_ref().context("No daily data available")?;
        let sunsets = daily.sunset.as_ref().context("No sunset in daily data")?;
        let result: Vec<_> = sunsets
            .iter()
            .map(|&s| self.offset_to_datetime(s))
            .collect();
        trace!("dailySunset: {:?}", result);
        Ok(result)
    }

    pub fn daily_temperature_min(&self) -> Result<Vec<Temperature>> {
        let daily = self.daily.as_ref().context("No daily data available")?;
        let temps = daily
            .temperature_2m_min
            .as_ref()
            .context("No temperature_2m_min in daily data")?;
        let result: Vec<_> = temps
            .iter()
            .map(|&d| Temperature { degrees: d as f64 })
            .collect();
        trace!("dailyTemperatureMin: {} entries", result.len());
        Ok(result)
    }

    pub fn daily_temperature_max(&self) -> Result<Vec<Temperature>> {
        let daily = self.daily.as_ref().context("No daily data available")?;
        let temps = daily
            .temperature_2m_max
            .as_ref()
            .context("No temperature_2m_max in daily data")?;
        let result: Vec<_> = temps
            .iter()
            .map(|&d| Temperature { degrees: d as f64 })
            .collect();
        trace!("dailyTemperatureMax: {} entries", result.len());
        Ok(result)
    }
}
