use chrono_tz::America::New_York;
use clap::{Parser, Subcommand};
use config::Profile;
use serde_json::{Value, json};
use third_party_api::weather::{
    CurrentField, DailyField, HourlyField, WeatherForecast, WeatherForecastBuilder,
};

const TIME_FORMAT: &str = "%a %b %-d %-I:%M%p";

#[derive(Debug, Subcommand)]
pub enum AlmanacCommand {
    Now,
    Today,
    ThisWeek,
}

#[derive(Debug, Parser)]
pub struct AlmanacArgs {
    #[clap(subcommand)]
    pub command: AlmanacCommand,
}

pub async fn handle_almanac_command(
    args: AlmanacArgs,
    profile: Option<&Profile>,
) -> anyhow::Result<()> {
    let profile = profile.ok_or_else(|| anyhow::anyhow!("Almanac command requires the profile"))?;
    match args.command {
        AlmanacCommand::Now => now_handler(profile).await,
        AlmanacCommand::Today => today_handler(profile).await,
        AlmanacCommand::ThisWeek => this_week_handler(profile).await,
    }
}

fn format_current(forecast: &WeatherForecast) -> anyhow::Result<Value> {
    let time = forecast.current_time()?;
    let temp = forecast.current_temperature()?;
    let apparent_temp = forecast.current_apparent_temperature()?;
    let weather = forecast.current_weather_code()?;

    Ok(json!({
        "time": time.with_timezone(&New_York).format(TIME_FORMAT).to_string(),
        "temperature": temp.as_fahrenheit(),
        "feels_like": apparent_temp.as_fahrenheit(),
        "weather_description": weather.description(),
    }))
}

fn format_hourly(forecast: &WeatherForecast) -> anyhow::Result<Vec<Value>> {
    let temps = forecast.hourly_temperatures()?;
    let weather_codes = forecast.hourly_weather_codes()?;

    Ok(weather_codes
        .iter()
        .zip(temps.iter())
        .map(|((dt, wmo), temp)| {
            json!({
                "time": dt.with_timezone(&New_York).format(TIME_FORMAT).to_string(),
                "temperature": temp.as_fahrenheit(),
                "weather_description": wmo.description(),
            })
        })
        .collect())
}

fn format_daily(forecast: &WeatherForecast) -> anyhow::Result<Vec<Value>> {
    let weather_codes = forecast.daily_weather_codes()?;
    let sunrise = forecast.daily_sunrise()?;
    let sunset = forecast.daily_sunset()?;
    let temp_min = forecast.daily_temperature_min()?;
    let temp_max = forecast.daily_temperature_max()?;

    Ok(weather_codes
        .iter()
        .zip(sunrise.iter())
        .zip(sunset.iter())
        .zip(temp_min.iter())
        .zip(temp_max.iter())
        .map(|(((((dt, wmo), sunrise), sunset), temp_min), temp_max)| {
            json!({
                "time": dt.with_timezone(&New_York).format(TIME_FORMAT).to_string(),
                "weather_description": wmo.description(),
                "sunrise": sunrise.with_timezone(&New_York).format(TIME_FORMAT).to_string(),
                "sunset": sunset.with_timezone(&New_York).format(TIME_FORMAT).to_string(),
                "temperature_min": temp_min.as_fahrenheit(),
                "temperature_max": temp_max.as_fahrenheit(),
            })
        })
        .collect())
}

async fn now_handler(profile: &Profile) -> anyhow::Result<()> {
    let forecast = WeatherForecastBuilder::new(profile.latitude, profile.longitude)
        .current([
            CurrentField::Temperature,
            CurrentField::WeatherCode,
            CurrentField::ApparentTemperature,
        ])
        .send()
        .await?;

    let output = json!({ "current": format_current(&forecast)? });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn today_handler(profile: &Profile) -> anyhow::Result<()> {
    let forecast = WeatherForecastBuilder::new(profile.latitude, profile.longitude)
        .current([CurrentField::Temperature, CurrentField::WeatherCode])
        .hourly([HourlyField::Temperature, HourlyField::WeatherCode])
        .daily([
            DailyField::WeatherCode,
            DailyField::Sunrise,
            DailyField::Sunset,
            DailyField::TemperatureMin,
            DailyField::TemperatureMax,
        ])
        .send()
        .await?;

    let output = json!({
        "current": format_current(&forecast)?,
        "hourly": format_hourly(&forecast)?,
        "daily": format_daily(&forecast)?,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn this_week_handler(profile: &Profile) -> anyhow::Result<()> {
    let forecast = WeatherForecastBuilder::new(profile.latitude, profile.longitude)
        .daily([
            DailyField::WeatherCode,
            DailyField::Sunrise,
            DailyField::Sunset,
            DailyField::TemperatureMin,
            DailyField::TemperatureMax,
        ])
        .send()
        .await?;

    let output = json!({ "daily": format_daily(&forecast)? });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
