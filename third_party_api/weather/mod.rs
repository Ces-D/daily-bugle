mod openmeteo;

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::trace;
use std::collections::BTreeMap;
use strum_macros::{Display, EnumString};

pub use openmeteo::{
    CurrentField, DailyField, HourlyField, Temperature, WeatherForecast, WeatherForecastBuilder,
    WmoWeatherCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedMode {
    Current,
    Daily,
    Hourly,
}

pub struct WeatherForecastToolInputs {
    pub latitude: f64,
    pub longitude: f64,
    pub forecast_days: Option<u8>,
    pub mode: SupportedMode,
}

#[derive(Debug, Default)]
pub struct WeatherForecastEntry {
    pub weather: String,
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
    pub temperature_f: Option<String>,
    pub temperature_f_min: Option<String>,
    pub temperature_f_max: Option<String>,
}

pub type WeatherForecastToolResponse = BTreeMap<DateTime<Utc>, WeatherForecastEntry>;

pub async fn weather_forecast_tool(
    inputs: WeatherForecastToolInputs,
) -> Result<WeatherForecastToolResponse> {
    trace!("Calling weather forecast tool.");

    let builder = match inputs.mode {
        SupportedMode::Current => WeatherForecastBuilder::new(
            inputs.latitude,
            inputs.longitude,
            inputs.forecast_days.unwrap_or(1),
        )
        .current([CurrentField::Temperature, CurrentField::WeatherCode]),
        SupportedMode::Daily => WeatherForecastBuilder::new(
            inputs.latitude,
            inputs.longitude,
            inputs.forecast_days.unwrap_or(7),
        )
        .daily([
            DailyField::WeatherCode,
            DailyField::Sunrise,
            DailyField::Sunset,
            DailyField::TemperatureMin,
            DailyField::TemperatureMax,
        ]),
        SupportedMode::Hourly => WeatherForecastBuilder::new(
            inputs.latitude,
            inputs.longitude,
            inputs.forecast_days.unwrap_or(1),
        )
        .hourly([HourlyField::Temperature, HourlyField::WeatherCode]),
    };

    let forecast = builder.send().await?;
    let mut data = WeatherForecastToolResponse::new();

    match inputs.mode {
        SupportedMode::Current => {
            data.insert(
                forecast.current_time()?,
                WeatherForecastEntry {
                    temperature_f: Some(forecast.current_temperature()?.as_fahrenheit()),
                    weather: forecast.current_weather_code()?.description().to_string(),
                    ..Default::default()
                },
            );
        }
        SupportedMode::Daily => {
            let daily_weather_codes = forecast.daily_weather_codes()?;
            let daily_sunrise = forecast.daily_sunrise()?;
            let daily_sunset = forecast.daily_sunset()?;
            let temperature_min = forecast.daily_temperature_min()?;
            let temperature_max = forecast.daily_temperature_max()?;

            for (i, (day, weather_code)) in daily_weather_codes.iter().enumerate() {
                data.insert(
                    *day,
                    WeatherForecastEntry {
                        weather: weather_code.description().to_string(),
                        sunrise: daily_sunrise.get(i).map(|dt| dt.to_rfc3339()),
                        sunset: daily_sunset.get(i).map(|dt| dt.to_rfc3339()),
                        temperature_f_min: temperature_min.get(i).map(|t| t.as_fahrenheit()),
                        temperature_f_max: temperature_max.get(i).map(|t| t.as_fahrenheit()),
                        ..Default::default()
                    },
                );
            }
        }
        SupportedMode::Hourly => {
            let hourly_weather_codes = forecast.hourly_weather_codes()?;
            let hourly_temperatures = forecast.hourly_temperatures()?;

            for (i, (time, weather_code)) in hourly_weather_codes.iter().enumerate() {
                data.insert(
                    *time,
                    WeatherForecastEntry {
                        weather: weather_code.description().to_string(),
                        temperature_f: hourly_temperatures.get(i).map(|t| t.as_fahrenheit()),
                        ..Default::default()
                    },
                );
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// New York City coordinates
    const NYC_LAT: f64 = 40.7128;
    const NYC_LON: f64 = -74.0060;

    fn inputs(mode: SupportedMode) -> WeatherForecastToolInputs {
        WeatherForecastToolInputs {
            latitude: NYC_LAT,
            longitude: NYC_LON,
            forecast_days: Some(1),
            mode,
        }
    }

    #[tokio::test]
    async fn current_mode_returns_single_entry_with_temperature_and_weather() {
        let data = weather_forecast_tool(inputs(SupportedMode::Current))
            .await
            .expect("current forecast failed");

        assert_eq!(
            data.len(),
            1,
            "Current mode should return exactly one entry"
        );

        let (_, entry) = data.iter().next().unwrap();
        assert!(
            !entry.weather.is_empty(),
            "Weather description should not be empty"
        );
        let temp = entry
            .temperature_f
            .as_ref()
            .expect("Current entry should have temperature_f");
        assert!(
            temp.ends_with("°F"),
            "Temperature should end with °F: {temp}"
        );
        assert!(entry.sunrise.is_none());
        assert!(entry.sunset.is_none());
        assert!(entry.temperature_f_min.is_none());
        assert!(entry.temperature_f_max.is_none());
    }

    #[tokio::test]
    async fn daily_mode_returns_seven_days_with_all_fields() {
        let data = weather_forecast_tool(inputs(SupportedMode::Daily))
            .await
            .expect("daily forecast failed");

        assert_eq!(data.len(), 7, "Daily mode should return 7 entries");

        for (day, entry) in &data {
            assert!(
                !entry.weather.is_empty(),
                "Day {day}: weather description should not be empty"
            );
            let sunrise = entry
                .sunrise
                .as_ref()
                .expect(&format!("Day {day}: should have sunrise"));
            let sunset = entry
                .sunset
                .as_ref()
                .expect(&format!("Day {day}: should have sunset"));
            assert!(
                sunrise < sunset,
                "Day {day}: sunrise {sunrise} should be before sunset {sunset}"
            );
            let min = entry
                .temperature_f_min
                .as_ref()
                .expect(&format!("Day {day}: should have temperature_f_min"));
            let max = entry
                .temperature_f_max
                .as_ref()
                .expect(&format!("Day {day}: should have temperature_f_max"));
            assert!(min.ends_with("°F"), "Day {day}: min temp format: {min}");
            assert!(max.ends_with("°F"), "Day {day}: max temp format: {max}");
            assert!(
                entry.temperature_f.is_none(),
                "Day {day}: should not have temperature_f"
            );
        }
    }

    #[tokio::test]
    async fn hourly_mode_returns_entries_with_temperature_and_weather() {
        let data = weather_forecast_tool(inputs(SupportedMode::Hourly))
            .await
            .expect("hourly forecast failed");

        assert!(
            data.len() > 24,
            "Hourly mode should return more than 24 entries, got {}",
            data.len()
        );

        for (time, entry) in &data {
            assert!(
                !entry.weather.is_empty(),
                "Hour {time}: weather description should not be empty"
            );
            let temp = entry
                .temperature_f
                .as_ref()
                .expect(&format!("Hour {time}: should have temperature_f"));
            assert!(temp.ends_with("°F"), "Hour {time}: temp format: {temp}");
            assert!(entry.sunrise.is_none());
            assert!(entry.sunset.is_none());
            assert!(entry.temperature_f_min.is_none());
            assert!(entry.temperature_f_max.is_none());
        }
    }

    #[tokio::test]
    async fn print_current_forecast() {
        let data = weather_forecast_tool(inputs(SupportedMode::Current))
            .await
            .expect("current forecast failed");
        for (time, entry) in &data {
            println!("{time}: {entry:?}");
        }
    }

    #[tokio::test]
    async fn print_daily_forecast() {
        let data = weather_forecast_tool(inputs(SupportedMode::Daily))
            .await
            .expect("daily forecast failed");
        for (time, entry) in &data {
            println!("{time}: {entry:?}");
        }
    }

    #[tokio::test]
    async fn print_hourly_forecast() {
        let data = weather_forecast_tool(inputs(SupportedMode::Hourly))
            .await
            .expect("hourly forecast failed");
        for (time, entry) in &data {
            println!("{time}: {entry:?}");
        }
    }

    #[test]
    fn wmo_weather_code_descriptions() {
        let common_codes = [
            (0, "Cloud development not observed"),
            (3, "Clouds generally forming or developing"),
            (51, "Drizzle, not freezing, continuous"),
            (61, "Rain, not freezing, continuous"),
            (71, "Continuous snowflakes"),
            (95, "Thunderstorm"),
        ];
        for (code, expected_prefix) in common_codes {
            let wmo = WmoWeatherCode { code };
            let desc = wmo.description();
            assert!(
                desc.starts_with(expected_prefix),
                "Code {code}: expected description starting with '{expected_prefix}', got '{desc}'"
            );
        }

        let unknown = WmoWeatherCode { code: 255 };
        assert_eq!(unknown.description(), "");
    }

    #[test]
    fn temperature_display() {
        let temp = Temperature { degrees: 72.5 };
        assert_eq!(temp.as_fahrenheit(), "72.5°F");
        assert_eq!(format!("{temp}"), "72.5°F");

        let cold = Temperature { degrees: -10.0 };
        assert_eq!(cold.as_fahrenheit(), "-10.0°F");
    }

    #[test]
    fn supported_mode_from_string() {
        use std::str::FromStr;
        assert_eq!(
            SupportedMode::from_str("current").unwrap(),
            SupportedMode::Current
        );
        assert_eq!(
            SupportedMode::from_str("daily").unwrap(),
            SupportedMode::Daily
        );
        assert_eq!(
            SupportedMode::from_str("hourly").unwrap(),
            SupportedMode::Hourly
        );
        assert!(SupportedMode::from_str("invalid").is_err());
    }
}
