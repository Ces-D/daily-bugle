use clap::builder::EnumValueParser;
use clap::{Arg, ArgAction, Command};

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
    Command::new("weather")
        .about("access the weather client")
        .arg(
            Arg::new("type")
                .short('t')
                .help("specify the highlight of the weather client")
                .action(ArgAction::Set)
                .value_parser(EnumValueParser::<WeatherType>::new())
                .default_value("current"),
        )
}

// TODO: implement the handler
