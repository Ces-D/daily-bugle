use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{UBER_ENGINEERING_BLOG_STORAGE_CONSTANT, UBER_ENGINEERING_BLOG_URL, UBER_ROOT_URL},
    xml::request_url_document_text,
};
use anyhow::Result;
use local_storage::key::StorageKey;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, HeaderMap, HeaderValue};
use scraper::Selector;

async fn request_uber_engineering_page(page: u32) -> Result<String> {
    let mut default_header = HeaderMap::new();
    default_header.insert(
        ACCEPT,
        HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .unwrap(),
    );
    default_header.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_str("gzip, deflate").unwrap(),
    );
    default_header.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_str("en-US,en;q=0.5").unwrap(),
    );
    let res = request_url_document_text(
        &format!("{}{}", UBER_ENGINEERING_BLOG_URL, page),
        Some(default_header),
    )
    .await?;
    Ok(res)
}

fn parse_uber_engineering_page(html: &str) -> Result<ScrapedEngineeringItems> {
    let html = scraper::Html::parse_document(html);
    let articles_selector = Selector::parse("div.bd.gw.i9.ja.jb > div > a").unwrap();
    let mut entries: ScrapedEngineeringItems = Vec::new();
    for element in html.select(&articles_selector) {
        let mut url = element.attr("href").unwrap().to_string();
        if url.starts_with("/") {
            url = format!("{}{}", UBER_ROOT_URL, url);
        }
        let title_selector = Selector::parse("h2").unwrap();
        let title = element.select(&title_selector).last().unwrap().inner_html();
        entries.push(ScrapedEngineeringItem {
            title,
            url,
            ..Default::default()
        });
    }
    Ok(entries)
}

pub async fn scrape_uber_engineering_blog() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(UBER_ENGINEERING_BLOG_STORAGE_CONSTANT).await {
        Some(item) => Ok(item),
        None => {
            let mut entries: ScrapedEngineeringItems = Vec::new();
            for page in 1..=10 {
                let res = request_uber_engineering_page(page).await?;
                let parsed = parse_uber_engineering_page(&res)?;
                entries.extend(parsed);
            }
            let storage_key = StorageKey::new(UBER_ENGINEERING_BLOG_STORAGE_CONSTANT, None, None);
            local_storage::write_item_to_storage(storage_key, &entries).await;
            Ok(entries)
        }
    }
}
