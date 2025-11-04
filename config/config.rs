pub mod configuration;
use anyhow::{Context, bail};
use log::info;
use std::path::PathBuf;

const DAILY_BUGLE_CONFIG_VAR: &str = "DAILY_BUGLE_CONFIG";
const CONFIG_DIR: &str = "daily_bugle";
const CONFIG_FILE: &str = "config.toml";

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
                let config_dir = path.join(".config").join(CONFIG_DIR);
                if !config_dir.exists() {
                    info!("Creating config directory: {}", config_dir.display());
                    std::fs::create_dir_all(config_dir)
                        .with_context(|| "Creating config directory")?;
                }
                Ok(path.join(".config").join(CONFIG_DIR))
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

pub fn read_config_file() -> anyhow::Result<configuration::Config> {
    let location = config_location()?.join(CONFIG_FILE);
    if location.exists() && location.is_file() {
        let content =
            std::fs::read_to_string(location).with_context(|| "Failed to read config file")?;
        let config = toml::from_str::<configuration::Config>(&content)
            .with_context(|| "Invalid toml in config file")?;
        Ok(config)
    } else {
        bail!(
            "Unable to open config file: {}",
            location
                .to_str()
                .unwrap_or_else(|| "unknown config location")
        )
    }
}
