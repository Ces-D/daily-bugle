use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{AWS_ENGINEERING_BLOG_SITEMAP_URL, AWS_ENGINEERING_BLOG_STORAGE_CONSTANT},
    xml::{XMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

#[derive(Default)]
struct AWSEngineeringSitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for AWSEngineeringSitemap {
    fn start(&mut self, name: &[u8]) -> Result<()> {
        match name {
            b"url" => {
                self.current_item = Some(ScrapedEngineeringItem::default());
            }
            b"loc" => {
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
            b"loc" => {
                if let Some(url) = &mut self.current_item {
                    match self.current_element.as_str() {
                        "loc" => url.url = self.current_text.clone(),
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

pub async fn scrape_aws_engineering_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(AWS_ENGINEERING_BLOG_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(AWS_ENGINEERING_BLOG_SITEMAP_URL, None).await?;
            let reader = Reader::from_str(&res);
            let handler = AWSEngineeringSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(AWS_ENGINEERING_BLOG_STORAGE_CONSTANT, None, Some(7));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}
