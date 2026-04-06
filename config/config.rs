use anyhow::{Context, bail};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DAILY_BUGLE_CONFIG_VAR: &str = "DAILY_BUGLE_CONFIG";
const PROJECT_NAME: &str = "daily_bugle";
const CONFIG_FILE: &str = "config.toml";

#[derive(Serialize, Deserialize, Clone)]
pub struct News {
    pub api_key: String,
    pub sources: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub known_as: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Path to the credentials file for google api
    pub google_calendar_credentials_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub news: News,
    pub profile: Vec<Profile>,
    pub openai_api_key: Option<String>,
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
                let config_dir = path.join(".config").join(PROJECT_NAME);
                if !config_dir.exists() {
                    info!("Creating config directory: {}", config_dir.display());
                    std::fs::create_dir_all(config_dir.clone())
                        .with_context(|| "Creating config directory")?;
                }
                Ok(config_dir)
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

pub fn application_storage(temporary: bool) -> anyhow::Result<PathBuf> {
    let path = std::env::home_dir().expect("Unable to locate home directory");
    let extension = match temporary {
        true => "state",
        false => "share",
    };
    let storage_dir = path.join(".local").join(extension).join(PROJECT_NAME);
    if !storage_dir.exists() {
        info!(
            "Creating application storage dir: {}",
            storage_dir.display()
        );
        std::fs::create_dir_all(storage_dir.clone())
            .with_context(|| "Creating application storage")?;
    }
    Ok(storage_dir)
}

pub fn read_config_file() -> anyhow::Result<Config> {
    let location = config_location()?.join(CONFIG_FILE);
    if location.exists() && location.is_file() {
        let content =
            std::fs::read_to_string(location).with_context(|| "Failed to read config file")?;
        let config =
            toml::from_str::<Config>(&content).with_context(|| "Invalid toml in config file")?;
        Ok(config)
    } else {
        bail!(
            "Unable to open config file: {}",
            location.to_str().unwrap_or("unknown config location")
        )
    }
}
