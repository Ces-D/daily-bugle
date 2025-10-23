use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{GOOGLE_DEVELOPER_BLOGS_SITEMAP_URL, GOOGLE_DEVELOPER_BLOGS_STORAGE_CONSTANT},
    xml::{XMLHandler, naive_date_to_utc, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use chrono::NaiveDate;
use local_storage::key::StorageKey;
use quick_xml::Reader;

#[derive(Default)]
struct GoogleDevelopersSitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for GoogleDevelopersSitemap {
    fn start(&mut self, name: &[u8]) -> Result<()> {
        match name {
            b"url" => {
                self.current_item = Some(ScrapedEngineeringItem::default());
            }
            b"loc" | b"lastmod" => {
                self.current_element = String::from_utf8_lossy(name.as_ref()).to_string();
                self.current_text.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn text(&mut self, txt: &str) -> Result<()> {
        if !self.current_element.is_empty() {
            self.current_text.push_str(txt.trim());
        }
        Ok(())
    }

    fn end(&mut self, name: &[u8]) -> Result<()> {
        match name {
            b"url" => {
                if let Some(url) = self.current_item.take() {
                    self.items.push(url);
                }
            }
            b"loc" | b"lastmod" => {
                if let Some(url) = &mut self.current_item {
                    match self.current_element.as_str() {
                        "loc" => url.url = self.current_text.clone(),
                        "lastmod" => {
                            if let Ok(d) = NaiveDate::parse_from_str(&self.current_text, "%Y-%m-%d")
                            {
                                url.updated = Some(naive_date_to_utc(d));
                            }
                        }
                        _ => {}
                    }
                }
                self.current_element.clear();
                self.current_text.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn items(self) -> ScrapedEngineeringItems {
        self.items
    }
}

pub async fn scrape_google_developer_blogs_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(GOOGLE_DEVELOPER_BLOGS_STORAGE_CONSTANT) {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(GOOGLE_DEVELOPER_BLOGS_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = GoogleDevelopersSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key =
                StorageKey::new(GOOGLE_DEVELOPER_BLOGS_STORAGE_CONSTANT, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items);
            Ok(items)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::naive_date_to_utc;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_xml_urls() {
        let xml = r#"<url>
<loc>https://developers.googleblog.com/en/</loc>
<lastmod>2025-10-02</lastmod>
</url>
<url>
<loc>https://developers.googleblog.com/en/join-us-online-from-23-27-october-for-passkeys-week/</loc>
<lastmod>2024-04-30</lastmod>
</url>
<url>
<loc>https://developers.googleblog.com/en/people-of-ai-season-2/</loc>
<lastmod>2024-03-22</lastmod>
</url>
<url>
<loc>https://developers.googleblog.com/en/save-the-date-for-firebases-first-demo-day/</loc>
<lastmod>2024-04-30</lastmod>
</url>
<url>
<loc>https://developers.googleblog.com/en/how-machine-learning-gde-henry-ruiz-is-inspired-by-resilience-in-his-community/</loc>
<lastmod>2024-03-22</lastmod>
</url>
<url>
<loc>https://developers.googleblog.com/en/mediapipe-on-device-text-to-image-generation-solution-now-available-for-android-developers/</loc>
<lastmod>2024-03-22</lastmod>
</url>"#;
        let reader = Reader::from_str(xml);
        let handler = GoogleDevelopersSitemap::default();
        let entries = parse_xml_with(reader, handler).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 6);
        let first = entries.first().unwrap();
        assert_eq!(first.url, "https://developers.googleblog.com/en/");
        assert!(first.updated.is_some());
        let last = entries.last().unwrap();
        assert_eq!(
            last.url,
            "https://developers.googleblog.com/en/mediapipe-on-device-text-to-image-generation-solution-now-available-for-android-developers/"
        );
        assert_eq!(
            last.updated,
            Some(naive_date_to_utc(
                NaiveDate::parse_from_str("2024-03-22", "%Y-%m-%d").unwrap()
            ))
        )
    }
}
