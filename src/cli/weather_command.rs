use crate::api_client::weather;
use clap::builder::EnumValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command};

use super::config::CliConfig;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum WeatherType {
    CURRENT,
}

impl clap::ValueEnum for WeatherType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::CURRENT]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::CURRENT => clap::builder::PossibleValue::new("current"),
        })
    }
}

pub fn create_command() -> Command {
    Command::new("weather").about("access the weather client").arg(
        Arg::new("type")
            .short('t')
            .help("specify the highlight of the weather client")
            .action(ArgAction::Set)
            .value_parser(EnumValueParser::<WeatherType>::new())
            .default_value("current"),
    )
}

// TODO: the handle_commands should return a ITerminalDisplay not Results
pub async fn handle_command(
    matches: &ArgMatches,
    config: CliConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let weather_type: WeatherType = *matches.get_one("type").expect("type is required");
    if weather_type == WeatherType::CURRENT {
        if config.api_keys.weather.is_some() && config.weather.current_location.is_some() {
            let current_weather = weather::get_current_weather(
                config.weather.current_location.unwrap(),
                config.api_keys.weather.unwrap(),
            )
            .await?;
        } else {

        }
    }
    Ok(())
}
