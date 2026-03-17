use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Weather {
    pub api_key: String,
    pub postal_code: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct News {
    pub api_key: String,
    pub sources: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleCalender {
    /// Path to the credentials file for google api
    #[serde(deserialize_with = "crate::path::deserialize_expanded_path")]
    pub credentials_file: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    /// Connection string to the surrealdb database
    pub connection_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub weather: Option<Weather>,
    pub news: Option<News>,
    pub google_calendar: Option<GoogleCalender>,
    pub database: Database,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            weather: Default::default(),
            news: Default::default(),
            google_calendar: Default::default(),
            database: Database {
                connection_url: "localhost:8000".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::expand_path;
    use std::env;

    #[test]
    fn test_expand_path_with_tilde() {
        let home = env::var("HOME").expect("HOME env var should be set");

        // Test ~ alone
        let result = expand_path("~");
        assert_eq!(result, PathBuf::from(home.clone()));

        // Test ~/path
        let result = expand_path("~/Documents/file.txt");
        assert_eq!(
            result,
            PathBuf::from(format!("{}/Documents/file.txt", home))
        );
    }

    #[test]
    fn test_expand_path_with_home_var() {
        let home = env::var("HOME").expect("HOME env var should be set");

        let result = expand_path("$HOME/Documents/file.txt");
        assert_eq!(
            result,
            PathBuf::from(format!("{}/Documents/file.txt", home))
        );
    }

    #[test]
    fn test_expand_path_no_expansion() {
        // Absolute path should remain unchanged
        let result = expand_path("/usr/local/bin");
        assert_eq!(result, PathBuf::from("/usr/local/bin"));

        // Relative path should remain unchanged
        let result = expand_path("relative/path");
        assert_eq!(result, PathBuf::from("relative/path"));
    }

    #[test]
    fn test_expand_path_undefined_var() {
        // Should keep the original $VAR if not found
        let result = expand_path("$UNDEFINED_VAR/file.txt");
        assert_eq!(result, PathBuf::from("$UNDEFINED_VAR/file.txt"));
    }

    #[test]
    fn test_expand_path_dollar_without_var() {
        // A lone $ should remain
        let result = expand_path("price:$/file.txt");
        assert_eq!(result, PathBuf::from("price:$/file.txt"));
    }

    #[test]
    fn test_deserialize_google_calendar() {
        let toml_str = r#"
            credentials_file = "~/credentials.json"
        "#;

        let calendar: GoogleCalender = toml::from_str(toml_str).expect("Failed to deserialize");
        let home = env::var("HOME").expect("HOME env var should be set");

        assert_eq!(
            calendar.credentials_file,
            PathBuf::from(format!("{}/credentials.json", home))
        );
    }
}
