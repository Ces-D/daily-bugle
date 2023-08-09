use crate::api_client::weather::Coordinates;
use directories;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiKeys {
    pub weather: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct WeatherConfig {
    pub current_location: Option<Coordinates>,
}

#[derive(Serialize, Deserialize)]
pub struct CliConfig {
    pub api_keys: ApiKeys,
    pub weather: WeatherConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            api_keys: ApiKeys { weather: None },
            weather: WeatherConfig { current_location: None },
        }
    }
}

static CONFIG_FILE_NAME: &str = "daily_bugle.toml";

pub fn read_config() -> CliConfig {
    let project_dirs = directories::ProjectDirs::from("xyz", "daily-bugle", "Daily Bugle");
    match project_dirs {
        Some(project_dirs) => {
            let config_file_exists = project_dirs.config_dir().join(CONFIG_FILE_NAME).exists();
            if config_file_exists {
                let config_file = project_dirs.config_dir().join(CONFIG_FILE_NAME);
                let config_file_contents = std::fs::read_to_string(config_file);
                match config_file_contents {
                    Ok(config_file_contents) => {
                        toml::from_str::<CliConfig>(&config_file_contents).unwrap_or_default()
                    },
                    Err(e) => {
                        eprintln!("Error reading config file: {}", e);
                        CliConfig::default()
                    },
                }
            } else {
                CliConfig::default()
            }
        },
        None => CliConfig::default(),
    }
}
