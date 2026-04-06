use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Serde helpers for DateTime<Utc> as epoch seconds ──

mod epoch_secs {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i64(dt.timestamp())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        let secs = i64::deserialize(d)?;
        DateTime::from_timestamp(secs, 0)
            .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
    }
}

mod epoch_secs_opt {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(dt: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error> {
        match dt {
            Some(dt) => s.serialize_some(&dt.timestamp()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<DateTime<Utc>>, D::Error> {
        let opt = Option::<i64>::deserialize(d)?;
        match opt {
            Some(secs) => DateTime::from_timestamp(secs, 0)
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom("invalid timestamp")),
            None => Ok(None),
        }
    }
}

// ── Rating ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

// ── Category ──

#[derive(Debug, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "epoch_secs")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CategoryInsert {
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "epoch_secs_opt")]
    pub created_at: Option<DateTime<Utc>>,
}

// ── Item ──

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: i64,
    pub category_id: i64,
    pub front: String,
    pub back: String,
    #[serde(with = "epoch_secs")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ItemInsert {
    pub category_id: i64,
    pub front: String,
    pub back: String,
    #[serde(with = "epoch_secs_opt")]
    pub created_at: Option<DateTime<Utc>>,
}

// ── Item State (FSRS scheduling) ──

#[derive(Debug, Deserialize)]
pub struct ItemState {
    pub item_id: i64,
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    #[serde(with = "epoch_secs")]
    pub due_at: DateTime<Utc>,
    #[serde(with = "epoch_secs_opt")]
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub reps: i64,
    pub lapses: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemStateInsert {
    pub item_id: i64,
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    #[serde(with = "epoch_secs")]
    pub due_at: DateTime<Utc>,
    #[serde(with = "epoch_secs_opt")]
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub reps: i64,
    pub lapses: i64,
}
