use chrono::{DateTime, Utc};

mod constant;
pub mod hackernews;
pub mod lucumr;
pub mod mdn;
pub mod openai;
pub mod time_out;
mod xml;

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ScrapedEngineeringItem {
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
}

pub type ScrapedEngineeringItems = Vec<ScrapedEngineeringItem>;
