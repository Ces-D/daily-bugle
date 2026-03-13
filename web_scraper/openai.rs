use crate::{
    ScrapedEngineeringItems,
    constant::{OPENAI_SITEMAP_STORAGE_CONSTANT, OPENAI_SITEMAP_URL},
    xml::{CommonXMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

pub async fn scrape_openai_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(OPENAI_SITEMAP_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(OPENAI_SITEMAP_URL, None).await?;
            let reader = Reader::from_str(&res);
            let items = parse_xml_with(reader, CommonXMLHandler::default())?;
            let storage_key = StorageKey::new(OPENAI_SITEMAP_STORAGE_CONSTANT, None, Some(7 * 24));
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
        let entries = parse_xml_with(reader, CommonXMLHandler::default())
            .expect("Failed to parse xml content");
        assert_eq!(entries.len(), 2);
        let first = entries.first().unwrap();
        assert_eq!(
            first.url,
            "https://openai.com/index/introducing-chatgpt-atlas/"
        );
        assert!(first.updated.is_some());
    }
}
