use anyhow::Context;
use config::local_storage_dir_location;
use log::info;

mod important_date;

const REMINDERS_FILE: &str = "reminders.toml";

pub fn read_reminders_file() -> anyhow::Result<important_date::Reminders> {
    let location = local_storage_dir_location().join(REMINDERS_FILE);
    if location.exists() && location.is_file() {
        let content =
            std::fs::read_to_string(location).with_context(|| "Failed to read config file")?;
        let config = toml::from_str::<important_date::Reminders>(&content)
            .with_context(|| "Invalid toml in config file")?;
        Ok(config)
    } else {
        info!("Creating default reminders file at {}", location.display());
        let default_dates = important_date::defaults()?;
        let toml_content = toml::to_string_pretty(&default_dates)
            .with_context(|| "Failed to serialize default dates to TOML")?;
        std::fs::write(&location, toml_content)
            .with_context(|| "Failed to write default reminders file")?;
        Ok(default_dates)
    }
}
