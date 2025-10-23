pub mod aws;
mod constant;
pub mod figma;
pub mod google;
pub mod hackernews;
pub mod lucumr;
pub mod mdn;
pub mod notion;
pub mod openai;
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

pub enum ScrapedSites {
    Aws,
    Figma,
    Google,
    Hackernews,
    ArminRonacher,
    Mdn,
    Notion,
    Openai,
    Timeout,
    Uber,
}

pub type ScrapedEngineeringItems = Vec<ScrapedEngineeringItem>;
