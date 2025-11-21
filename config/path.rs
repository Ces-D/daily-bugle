use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

/// Expands environment variables and tilde in path strings
/// Supports:
/// - `~` at the start -> user's home directory
/// - `$VAR` -> environment variable expansion
pub fn expand_path(path_str: &str) -> PathBuf {
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
pub fn deserialize_expanded_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path_str = String::deserialize(deserializer)?;
    Ok(expand_path(&path_str))
}
