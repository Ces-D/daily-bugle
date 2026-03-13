use crate::{
    ScrapedEngineeringItems,
    constant::{AWS_ENGINEERING_BLOG_SITEMAP_URL, AWS_ENGINEERING_BLOG_STORAGE_CONSTANT},
    xml::{CommonXMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

pub async fn scrape_aws_engineering_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(AWS_ENGINEERING_BLOG_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(AWS_ENGINEERING_BLOG_SITEMAP_URL, None).await?;
            let reader = Reader::from_str(&res);
            let items = parse_xml_with(reader, CommonXMLHandler::default())?;
            let storage_key =
                StorageKey::new(AWS_ENGINEERING_BLOG_STORAGE_CONSTANT, None, Some(7 * 24));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}
