use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{FIGMA_ENGINEERING_BLOG_STORAGE_CONSTANT, FIGMA_ENGINEERING_BLOG_URL},
    xml::request_url_document_text,
};
use anyhow::Result;
use local_storage::key::StorageKey;
use scraper::Selector;

pub async fn scrape_figma_engineering_blog() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(FIGMA_ENGINEERING_BLOG_STORAGE_CONSTANT) {
        Some(item) => Ok(item),
        None => {
            let res = request_url_document_text(FIGMA_ENGINEERING_BLOG_URL).await?;
            let html = scraper::Html::parse_document(&res);
            let engineering_blogs_selector =
                Selector::parse("section#more-engineering-blogs > div > ul > li > article")
                    .unwrap();
            let mut entries: ScrapedEngineeringItems = Vec::new();
            for element in html.select(&engineering_blogs_selector) {
                let content_selector = Selector::parse("div > div > a.fig-bqm9r8").unwrap();
                let content = element.select(&content_selector).last().unwrap();
                let url = content.attr("href").unwrap().to_string();
                let title_selector = Selector::parse("h3").unwrap();
                let title = content.select(&title_selector).last().unwrap().inner_html();
                let summary_selector = Selector::parse("footer p").unwrap();
                let summary = content
                    .select(&summary_selector)
                    .last()
                    .unwrap()
                    .inner_html();
                entries.push(ScrapedEngineeringItem {
                    title,
                    url,
                    summary: Some(summary),
                    ..Default::default()
                });
            }
            let storage_key =
                StorageKey::new(FIGMA_ENGINEERING_BLOG_STORAGE_CONSTANT, None, Some(14));
            local_storage::write_item_to_storage(storage_key, &entries);
            Ok(entries)
        }
    }
}
