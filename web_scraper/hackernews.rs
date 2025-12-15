use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{
        HACKER_NEWS_JOBS_STORAGE_CONSTANT, HACKER_NEWS_JOBS_URL, HACKER_NEWS_NEWS_STORAGE_CONSTANT,
        HACKER_NEWS_NEWS_URL,
    },
    xml::request_url_document_text,
};
use anyhow::Result;
use local_storage::key::StorageKey;
use scraper::Selector;

#[derive(Default)]
pub enum Page {
    #[default]
    First,
    Second,
}

fn create_url(url: &str, page: Page) -> String {
    let mut url = url.to_string();
    let params = match page {
        Page::First => format!("?p={}", 1),
        Page::Second => format!("?p={}", 2),
    };
    url.push_str(&params);
    url
}

pub async fn scrape_hackernews_news(page: Option<Page>) -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(HACKER_NEWS_NEWS_STORAGE_CONSTANT).await {
        Some(item) => Ok(item),
        None => {
            let res = request_url_document_text(
                &create_url(HACKER_NEWS_NEWS_URL, page.unwrap_or_default()),
                None,
            )
            .await?;
            let html = scraper::Html::parse_document(&res);
            let title_selector =
                Selector::parse("tr.athing.submission > td.title > span.titleline > a").unwrap();
            let mut entries: ScrapedEngineeringItems = Vec::new();
            for element in html.select(&title_selector) {
                let url = element.attr("href").unwrap().to_string();
                let title = element.inner_html();
                entries.push(ScrapedEngineeringItem {
                    title,
                    url,
                    ..Default::default()
                });
            }
            let storage_key =
                StorageKey::new(HACKER_NEWS_NEWS_STORAGE_CONSTANT, None, Some(1 * 24));
            local_storage::write_item_to_storage(storage_key, &entries).await;
            Ok(entries)
        }
    }
}

pub async fn scrape_hackernews_jobs(page: Option<Page>) -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(HACKER_NEWS_JOBS_STORAGE_CONSTANT).await {
        Some(item) => Ok(item),
        None => {
            let res = request_url_document_text(
                &create_url(HACKER_NEWS_JOBS_URL, page.unwrap_or_default()),
                None,
            )
            .await?;
            let html = scraper::Html::parse_document(&res);
            let title_selector =
                Selector::parse("tr.athing.submission > td.title > span.titleline > a").unwrap();
            let mut entries: ScrapedEngineeringItems = Vec::new();
            for element in html.select(&title_selector) {
                let url = element.attr("href").unwrap().to_string();
                let title = element.inner_html();
                entries.push(ScrapedEngineeringItem {
                    title,
                    url,
                    ..Default::default()
                });
            }
            let storage_key =
                StorageKey::new(HACKER_NEWS_JOBS_STORAGE_CONSTANT, None, Some(1 * 24));
            local_storage::write_item_to_storage(storage_key, &entries).await;
            Ok(entries)
        }
    }
}
