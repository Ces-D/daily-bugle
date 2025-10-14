use crate::constant::{HACKER_NEWS_JOBS_URL, HACKER_NEWS_NEWS_URL};
use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use scraper::Selector;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub enum Page {
    #[default]
    First,
    Second,
}

fn create_pagination_params(page: Page) -> String {
    match page {
        Page::First => format!("?p={}", 1),
        Page::Second => format!("?p={}", 2),
    }
}

async fn request_hackernews(url: &str, page: Page) -> Result<String> {
    let mut url = url.to_string();
    url.push_str(&create_pagination_params(page));
    let res = reqwest::get(&url).await?;
    if res.status() != StatusCode::OK {
        bail!("Failed request to {} - {}", url, res.status());
    } else {
        res.text()
            .await
            .with_context(|| "Failed to decode response body")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    title: String,
    link: String,
}

pub async fn get_hackernews_news(page: Option<Page>) -> Result<Vec<Entry>> {
    let res = request_hackernews(HACKER_NEWS_NEWS_URL, page.unwrap_or_default()).await?;
    let html = scraper::Html::parse_document(&res);
    let title_selector =
        Selector::parse("tr.athing.submission > td.title > span.titleline > a").unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    for element in html.select(&title_selector) {
        let link = element.attr("href").unwrap().to_string();
        let title = element.inner_html();
        entries.push(Entry { title, link });
    }
    Ok(entries)
}

pub async fn get_hackernews_jobs(page: Option<Page>) -> Result<Vec<Entry>> {
    let res = request_hackernews(HACKER_NEWS_JOBS_URL, page.unwrap_or_default()).await?;
    let html = scraper::Html::parse_document(&res);
    let title_selector =
        Selector::parse("tr.athing.submission > td.title > span.titleline > a").unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    for element in html.select(&title_selector) {
        let link = element.attr("href").unwrap().to_string();
        let title = element.inner_html();
        entries.push(Entry { title, link });
    }
    Ok(entries)
}
