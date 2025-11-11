use serde::{Deserialize, Serialize};

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
pub struct Database {
    pub connection_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub weather: Option<Weather>,
    pub google_calendar: Option<GoogleCalender>,
    pub database: Database,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            weather: Default::default(),
            google_calendar: Default::default(),
            database: Database {
                connection_url: "localhost:8000".to_string(),
            },
        }
    }
}
