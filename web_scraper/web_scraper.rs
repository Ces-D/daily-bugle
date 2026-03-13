pub mod aws;
mod constant;
pub mod deep_learning;
pub mod figma;
pub mod github;
pub mod google;
pub mod hackernews;
pub mod imperva;
pub mod lucumr;
pub mod mdn;
pub mod medium;
pub mod netflix;
pub mod notion;
pub mod nytimes;
pub mod openai;
pub mod square;
pub mod stripe;
pub mod time_out;
pub mod uber;
mod xml;

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ScrapedEngineeringItem {
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub published: Option<chrono::DateTime<chrono::Utc>>,
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
}

pub type ScrapedEngineeringItems = Vec<ScrapedEngineeringItem>;
