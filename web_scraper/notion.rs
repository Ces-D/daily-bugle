use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{NOTION_BLOG_SITEMAP_STORAGE_CONSTANT, NOTION_BLOG_SITEMAP_URL},
    xml::{XMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use local_storage::key::StorageKey;
use quick_xml::Reader;

#[derive(Default)]
struct NotionBlogSitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for NotionBlogSitemap {
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
                            if let Ok(dt) = &self.current_text.parse::<DateTime<Utc>>() {
                                url.updated = Some(dt.clone());
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

pub async fn scrape_notion_blog_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(NOTION_BLOG_SITEMAP_STORAGE_CONSTANT) {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(NOTION_BLOG_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = NotionBlogSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(NOTION_BLOG_SITEMAP_STORAGE_CONSTANT, None, Some(7));
            local_storage::write_item_to_storage(storage_key, &items);
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
<loc>https://www.notion.com/blog/ai-art-prompts</loc>
<lastmod>2024-10-06T20:22:23.809Z</lastmod>
<changefreq>yearly</changefreq>
<priority>0.4</priority>
</url>
<url>
<loc>https://www.notion.com/blog/its-national-coffee-day-grab-a-cup-and-some-templates</loc>
<lastmod>2024-09-30T22:58:45.507Z</lastmod>
<changefreq>yearly</changefreq>
<priority>0.4</priority>
</url>
<url>
<loc>https://www.notion.com/blog/how-i-learned-to-stop-worrying-and-love-ai</loc>
<lastmod>2024-09-30T16:53:29.263Z</lastmod>
<changefreq>yearly</changefreq>
<priority>0.4</priority>
</url>"#;
        let reader = Reader::from_str(xml);
        let handler = NotionBlogSitemap::default();
        let entries = parse_xml_with(reader, handler).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 3);
        let first = entries.first().unwrap();
        assert_eq!(first.url, "https://www.notion.com/blog/ai-art-prompts");
        assert!(first.updated.is_some());
    }
}
