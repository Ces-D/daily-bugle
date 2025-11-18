use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

/// Expands environment variables and tilde in path strings
/// Supports:
/// - `~` at the start -> user's home directory
/// - `$VAR` -> environment variable expansion
fn expand_path(path_str: &str) -> PathBuf {
    // Expand tilde at the start
    let after_tilde = if path_str.starts_with('~') {
        std::env::var("HOME")
            .ok()
            .map(|home| format!("{}{}", home, &path_str[1..]))
            .unwrap_or_else(|| path_str.to_string())
    } else {
        path_str.to_string()
    };

    // Expand $VAR environment variables
    let mut chars = after_tilde.chars().peekable();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !var_name.is_empty() {
                if let Ok(var_value) = std::env::var(&var_name) {
                    result.push_str(&var_value);
                } else {
                    // Keep original if variable not found
                    result.push('$');
                    result.push_str(&var_name);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }

    PathBuf::from(result)
}

/// Custom deserializer for PathBuf that expands environment variables and tilde
fn deserialize_expanded_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path_str = String::deserialize(deserializer)?;
    Ok(expand_path(&path_str))
}

#[derive(Serialize, Deserialize)]
pub struct Weather {
    pub api_key: String,
    pub postal_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct GoogleCalender {
    /// Path to the credentials file for google api
    #[serde(deserialize_with = "deserialize_expanded_path")]
    pub credentials_file: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    /// Connection string to the surrealdb database
    pub connection_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Resume {
    /// Path to the user resume
    #[serde(deserialize_with = "deserialize_expanded_path")]
    pub path: PathBuf,
    pub headings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Career {
    /// Resume related config
    pub resume: Resume,
    /// Path to the user profile summary
    #[serde(deserialize_with = "deserialize_expanded_path")]
    pub profile: PathBuf,
    /// Job description
    #[serde(deserialize_with = "deserialize_expanded_path")]
    pub job: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub weather: Option<Weather>,
    pub google_calendar: Option<GoogleCalender>,
    pub database: Database,
    pub career: Option<Career>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            weather: Default::default(),
            google_calendar: Default::default(),
            career: Default::default(),
            database: Database {
                connection_url: "localhost:8000".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_deserialize_resume() {
        let toml_str = r#"
            path = "~/Documents/resume.pdf"
            headings = ["Education", "Experience"]
        "#;

        let resume: Resume = toml::from_str(toml_str).expect("Failed to deserialize");
        let home = env::var("HOME").expect("HOME env var should be set");

        assert_eq!(
            resume.path,
            PathBuf::from(format!("{}/Documents/resume.pdf", home))
        );
        assert_eq!(resume.headings, vec!["Education", "Experience"]);
    }

    #[test]
    fn test_deserialize_career() {
        let toml_str = r#"
            profile = "~/Documents/profile.txt"
            job = "~/Documents/description.txt"

            [resume]
            path = "~/Documents/resume.pdf"
            headings = ["Skills"]
        "#;

        let career: Career = toml::from_str(toml_str).expect("Failed to deserialize");
        let home = env::var("HOME").expect("HOME env var should be set");

        assert_eq!(
            career.profile,
            PathBuf::from(format!("{}/Documents/profile.txt", home))
        );
        assert_eq!(
            career.resume.path,
            PathBuf::from(format!("{}/Documents/resume.pdf", home))
        );
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

    #[test]
    fn test_deserialize_full_config() {
        let toml_str = r#"
            [database]
            connection_url = "localhost:8000"

            [google_calendar]
            credentials_file = "~/creds.json"

            [career]
            profile = "$HOME/profile.txt"
            job = "~/job.txt"

            [career.resume]
            path = "~/resume.pdf"
            headings = ["Education"]
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to deserialize");
        let home = env::var("HOME").expect("HOME env var should be set");

        assert!(config.google_calendar.is_some());
        assert_eq!(
            config.google_calendar.unwrap().credentials_file,
            PathBuf::from(format!("{}/creds.json", home))
        );

        assert!(config.career.is_some());
        let career = config.career.unwrap();
        assert_eq!(
            career.profile,
            PathBuf::from(format!("{}/profile.txt", home))
        );
        assert_eq!(career.job, PathBuf::from(format!("{}/job.txt", home)));
        assert_eq!(
            career.resume.path,
            PathBuf::from(format!("{}/resume.pdf", home))
        );
    }
}
