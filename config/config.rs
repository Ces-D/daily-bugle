use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DAILY_BUGLE_CONFIG_VAR: &str = "DAILY_BUGLE_CONFIG";
const CONFIG_DIR: &str = "daily_bugle";
const CONFIG_FILE: &str = "config.toml";

#[derive(Serialize, Deserialize)]
pub struct Weather {
    pub api_key: String,
    pub postal_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleCalender {
    pub credentials_file: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub weather: Option<Weather>,
    pub google_calender: Option<GoogleCalender>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            weather: Default::default(),
            google_calender: Default::default(),
        }
    }
}

fn config_location() -> anyhow::Result<PathBuf> {
    match std::env::var(DAILY_BUGLE_CONFIG_VAR) {
        Ok(location) => {
            let mut path = PathBuf::new();
            path.push(location);
            Ok(path)
        }
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                let path = std::env::home_dir().expect("Unable to locate home directory");
                Ok(path.join(".config").join(CONFIG_DIR).join(CONFIG_FILE))
            }
            std::env::VarError::NotUnicode(os_string) => {
                bail!(
                    "Failed to interpret {} env var: {:?}",
                    DAILY_BUGLE_CONFIG_VAR,
                    os_string
                )
            }
        },
    }
}
pub fn local_storage_dir_location() -> PathBuf {
    let path = std::env::home_dir().expect("Unable to locate home directory");
    path.join(".local").join("state").join(CONFIG_DIR)
}

pub fn read_config_file() -> anyhow::Result<Config> {
    let config_location = config_location()?;
    if config_location.exists() && config_location.is_file() {
        let content = std::fs::read_to_string(config_location)
            .with_context(|| "Failed to read config file")?;
        let config =
            toml::from_str::<Config>(&content).with_context(|| "Invalid toml in config file")?;
        Ok(config)
    } else {
        bail!(
            "Unable to open config file: {}",
            config_location
                .to_str()
                .unwrap_or_else(|| "unknown config location")
        )
    }
}
