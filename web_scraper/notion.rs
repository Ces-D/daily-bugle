use crate::{
    ScrapedEngineeringItems,
    constant::{NOTION_BLOG_SITEMAP_STORAGE_CONSTANT, NOTION_BLOG_SITEMAP_URL},
    xml::{CommonXMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

pub async fn scrape_notion_blog_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(NOTION_BLOG_SITEMAP_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(NOTION_BLOG_SITEMAP_URL, None).await?;
            let reader = Reader::from_str(&res);
            let items = parse_xml_with(reader, CommonXMLHandler::default())?;
            let storage_key =
                StorageKey::new(NOTION_BLOG_SITEMAP_STORAGE_CONSTANT, None, Some(7 * 24));
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
        let entries = parse_xml_with(reader, CommonXMLHandler::default())
            .expect("Failed to parse xml content");
        assert_eq!(entries.len(), 3);
        let first = entries.first().unwrap();
        assert_eq!(first.url, "https://www.notion.com/blog/ai-art-prompts");
        assert!(first.updated.is_some());
    }
}
