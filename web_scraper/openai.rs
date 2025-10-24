use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{OPENAI_SITEMAP_STORAGE_CONSTANT, OPENAI_SITEMAP_URL},
    xml::{XMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

#[derive(Default)]
struct OpenAISitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for OpenAISitemap {
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
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&self.current_text)
                            {
                                url.updated = Some(dt.to_utc());
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

pub async fn scrape_openai_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(OPENAI_SITEMAP_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(OPENAI_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = OpenAISitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(OPENAI_SITEMAP_STORAGE_CONSTANT, None, Some(7));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml() {
        let xml = r#"
        <url>
        <loc>https://openai.com/index/introducing-chatgpt-atlas/</loc>
        <lastmod>2025-10-21T21:14:43.217Z</lastmod>
        </url>
        <url >
        <loc>https://openai.com/chatgpt/pricing/</loc>
        <lastmod>2025-10-21T21:03:39.390Z</lastmod>
        </url>"#;
        let reader = Reader::from_str(xml);
        let handler = OpenAISitemap::default();
        let entries = parse_xml_with(reader, handler).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 2);
        let first = entries.first().unwrap();
        assert_eq!(
            first.url,
            "https://openai.com/index/introducing-chatgpt-atlas/"
        );
        assert!(first.updated.is_some());
    }
}
