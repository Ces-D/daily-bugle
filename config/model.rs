use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct News {
    pub api_key: String,
    pub sources: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub known_as: String,
    /// Path to the credentials file for google api
    pub google_calendar_credentials_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub news: News,
    pub profile: Vec<Profile>,
}
