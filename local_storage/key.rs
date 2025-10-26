use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use std::{hash::Hasher, path::PathBuf};

/// A storage key that compares for equality based only on its `constant` field.
#[derive(Debug, Clone)]
pub struct StorageKey {
    pub constant: String,
    expires_on: chrono::DateTime<Utc>,
}

impl StorageKey {
    /// Helper function to create a new key.
    pub fn new(
        constant: &str,
        issued_at: Option<DateTime<Utc>>,
        lifetime_days: Option<i64>,
    ) -> Self {
        let now = match issued_at {
            Some(n) => n,
            None => Utc::now(),
        };
        let expires = now + Duration::days(lifetime_days.unwrap_or(3));
        Self {
            constant: sanitize(constant),
            expires_on: expires,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_on < Utc::now()
    }
}


/// Implement PartialEq to define custom equality logic.
/// Two keys are equal if their `constant` fields are equal.
impl PartialEq for StorageKey {
    fn eq(&self, other: &Self) -> bool {
        self.constant == other.constant
    }
}

/// Implement Eq since our equality logic is reflexive, symmetric, and transitive.
impl Eq for StorageKey {}

/// Implement Hash to be consistent with PartialEq.
/// The hash should only be derived from the `constant` field.
/// This allows the struct to be used correctly in HashMaps and HashSets.
impl std::hash::Hash for StorageKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.constant.hash(state);
    }
}

impl From<PathBuf> for StorageKey {
    fn from(value: PathBuf) -> Self {
        match value
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split_once("__")
        {
            Some((constant, expired)) => {
                let naive = NaiveDateTime::parse_from_str(expired, "%Y%m%d%H%M%S").unwrap();
                Self {
                    constant: constant.to_string(),
                    expires_on: DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc),
                }
            }
            None => {
                panic!("The length pathbuf is not written as expected")
            }
        }
    }
}

impl Into<PathBuf> for &StorageKey {
    fn into(self) -> PathBuf {
        let formatted_dt = self.expires_on.format("%Y%m%d%H%M%S").to_string();
        let s = format!("{}__{}", self.constant, formatted_dt);
        let mut path = PathBuf::new();
        path.push(s);
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
