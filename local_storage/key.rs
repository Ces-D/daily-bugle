use chrono::TimeZone;
use std::path::PathBuf;

#[derive(Debug, Hash, Clone)]
pub struct LocalStorageKey(String);

impl LocalStorageKey {
    /// Constant should be an element related to the type of item being stored
    pub fn new(constant: &str) -> Self {
        Self(sanitize(constant))
    }

    pub fn push_datetime(mut self, dt: chrono::DateTime<chrono_tz::Tz>) -> Self {
        let formatted_dt = format!("DT{}", dt.format("%Y%m%d%H%M%S").to_string());
        let mut key = self.0.clone();
        key.push_str("__");
        key.push_str(&formatted_dt);
        self.0 = key;
        self
    }

    /// Adding datetime adds uniqueness to the storage path but not used for comparing equivalence of cache key
    pub fn datetime(&self, timezone: chrono_tz::Tz) -> Option<chrono::DateTime<chrono_tz::Tz>> {
        let keys = self.0.split("__");
        let dt_key = keys.into_iter().find(|v| v.starts_with("DT"))?;
        let trimmed = &dt_key[2..]; // Remove "DT" prefix
        let naive_dt = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y%m%d%H%M%S").ok()?;
        timezone.from_local_datetime(&naive_dt).single()
    }
}

impl PartialEq for LocalStorageKey {
    fn eq(&self, other: &Self) -> bool {
        let o = self.0.split_once("__");
        let n = other.0.split_once("__");
        if o.is_some() && n.is_some() {
            o.unwrap().0 == n.unwrap().0 // We only care about the constant being identical
        } else if o.is_none() && n.is_none() {
            false
        } else if o.is_some() && n.is_none() || o.is_none() && n.is_some() {
            return false;
        } else {
            false
        }
    }
}

impl Eq for LocalStorageKey {}

impl From<PathBuf> for LocalStorageKey {
    fn from(value: PathBuf) -> LocalStorageKey {
        let s = value.file_stem().unwrap().to_str().unwrap();
        LocalStorageKey(s.to_string())
    }
}

impl Into<PathBuf> for &LocalStorageKey {
    fn into(self) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(&self.0);
        path.with_extension("dat");
        path
    }
}

/// Build a filename-safe key from (user, datetime).
///
/// Rules:
/// - Only allow [A-Za-z0-9._-] and replace everything else with '_'
/// - Join with a double underscore to avoid ambiguity
/// - Add a `.dat` extension for clarity (still part of the key)
fn sanitize(s: &str) -> String {
    let s = s.replace("__", "_");
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => c,
            _ => '_',
        })
        .collect()
}
