use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{MDN_SITEMAP_STORAGE_CONSTANT, MDN_SITEMAP_URL},
    xml::{XMLHandler, naive_date_to_utc, parse_xml_with},
};
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use local_storage::key::StorageKey;
use quick_xml::Reader;
use reqwest::StatusCode;
use std::io::Read;

async fn request_mdn_sitemap() -> Result<String> {
    let res = reqwest::get(MDN_SITEMAP_URL).await?;
    if res.status() != StatusCode::OK {
        bail!("Failed request to {} - {}", MDN_SITEMAP_URL, res.status());
    } else {
        let bytes = res
            .bytes()
            .await
            .with_context(|| "Failed to decode response body")?;
        // Decompress using flate2
        let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut xml = String::new();
        decoder.read_to_string(&mut xml)?;
        Ok(xml)
    }
}

#[derive(Default)]
struct MDNSitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for MDNSitemap {
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

pub async fn scrape_mdn_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(MDN_SITEMAP_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_mdn_sitemap().await?;
            let reader = Reader::from_str(&res);
            let handler = MDNSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(MDN_SITEMAP_STORAGE_CONSTANT, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
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
        <loc>https://developer.mozilla.org/en-US/</loc>
        </url>
        <url>
        <loc>https://developer.mozilla.org/en-US/docs/Games/Publishing_games/Game_promotion</loc>
        <lastmod>2025-07-11</lastmod>
        </url>
        <url>
        <loc>https://developer.mozilla.org/en-US/docs/Games/Techniques</loc>
        <lastmod>2025-07-11</lastmod>
        </url>
        <url>
        <loc>https://developer.mozilla.org/en-US/docs/Games/Techniques/2D_collision_detection</loc>
        <lastmod>2025-07-11</lastmod>
        </url>"#;
        let reader = Reader::from_str(xml);
        let handler = MDNSitemap::default();
        let entries = parse_xml_with(reader, handler).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 4);
        let first = entries.first().unwrap();
        assert_eq!(first.url, "https://developer.mozilla.org/en-US/");
        assert!(first.updated.is_none());
        let last = entries.last().unwrap();
        assert_eq!(
            last.url,
            "https://developer.mozilla.org/en-US/docs/Games/Techniques/2D_collision_detection"
        );
        assert_eq!(
            last.updated,
            Some(naive_date_to_utc(
                NaiveDate::parse_from_str("2025-07-11", "%Y-%m-%d").unwrap()
            ))
        )
    }
}
