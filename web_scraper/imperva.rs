use crate::{
    ScrapedEngineeringItem, ScrapedEngineeringItems,
    constant::{
        IMPERVA_LEARN_APPLICATION_SECURITY_SITEMAP_URL, IMPERVA_LEARN_AVAILABILITY_SITEMAP_URL,
        IMPERVA_LEARN_DATA_SECURITY_SITEMAP_URL, IMPERVA_LEARN_DDOS_SITEMAP_URL,
        IMPERVA_LEARN_PERFORMANCE_SITEMAP_URL,
    },
    xml::{XMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use local_storage::key::StorageKey;
use log::trace;
use quick_xml::Reader;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, COOKIE, HeaderMap, HeaderValue};

async fn request_imperva_sitemap(url: &str) -> Result<String> {
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
    default_header.insert(COOKIE, HeaderValue::from_str("_impv_routing=d5133ded779c92f77089195c083e0646|44e043a86861b109997cc70e7a6eba67; nlbi_2439_3105325=XLBuaDqG2g+cKI/3kzEmpwAAAABOg5zxxYoqqWacy2vKvdv0; visid_incap_2439=5KwFznzLTuWuCAbNSWm5Hlz7+GgAAAAAQUIPAAAAAACXCjN3MkrQAvHFvbhBGViA; isReturningUser=true; drift_aid=9ef3c755-a971-4e0a-97cd-032197d66cf6; driftt_aid=9ef3c755-a971-4e0a-97cd-032197d66cf6; nlbi_2439=JQNNdHVOCWWE7us8kzEmpwAAAAA7LLjpit4F8wJ/YvyeLzH+; incap_ses_155_2439=4uPTbejLyBIpEA4zG6wmAnu9/2gAAAAAwS6ARl3IkPeNS+ilyHZXXg==; nlbi_2439_2147483392=b+nBNjX6YwP9hmqskzEmpwAAAABhmqvY+Z5l++WPW3azwhTr; reese84=3:IUMeGPelmvbGwtfdWBre8g==:MaOyHAqFl+pH1P1Ip1tK+L7fXWsAf7ZePLIEJDBQmNl5abpErHr0jLIp/+ux3UBMAtYBGRPum3VmFSMwF9wqh8rCjbUyp/Wsx4dK6XRf55G203fKkxbQULAQ6YnvZO4V+W8uTOL8JbR1/9vXz3kBUuMtm9zd20/tkO2Bqv62+GIWpOLMmw3YXnJvjaSflVtkxC9TdaBTzJUmlIpK/e5TF3SGfmVLpeVahlrvBWqyAefPE217W98OL+8QjJyujKML0m8mCVTDFFWiTulIizi+4zcErjwdM8h9mt7BrFmbZQ9jclCvD5Y4VuFsbZgOsNkfHT+SWChM2xpmVAgWCLCB7p2v2H6byzdK4bw6vlWPf5/ojJYfHqlgLiEmscgDywqrtLao+PX1xclMNCrvKxU366MZLJqq94U+N6jKovsudBUeQlf6JWFoKZyUS2SuSS5g3uzocagDyE8icww2vlBbJV39WomaCGt3eppCs/fk3QGOcYylq3lT4YCOK0JwIri0:1kJa7TwR5aFfr+8dxcXo/yfupvGzE6R1kTAAVmB8BKM=").unwrap());

    let res = request_url_document_text(url, Some(default_header)).await?;
    trace!("res: {}", res);
    Ok(res)
}

fn imperva_cache_constant(url: &str) -> String {
    match url {
        IMPERVA_LEARN_APPLICATION_SECURITY_SITEMAP_URL => {
            "imperva-application-security-sitemap".to_string()
        }
        IMPERVA_LEARN_AVAILABILITY_SITEMAP_URL => "imperva-availability-sitemap".to_string(),
        IMPERVA_LEARN_DATA_SECURITY_SITEMAP_URL => "imperva-data-security-sitemap".to_string(),
        IMPERVA_LEARN_DDOS_SITEMAP_URL => "imperva-ddos-sitemap".to_string(),
        IMPERVA_LEARN_PERFORMANCE_SITEMAP_URL => "imperva-performance-sitemap".to_string(),
        _ => "imperva-sitemap".to_string(),
    }
}

#[derive(Default)]
struct ImpervaSitemap {
    items: ScrapedEngineeringItems,
    current_item: Option<ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl XMLHandler<ScrapedEngineeringItems> for ImpervaSitemap {
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
                            let dt = DateTime::parse_from_rfc3339(&self.current_text)?;
                            let utc_dt = dt.with_timezone(&Utc);
                            url.updated = Some(utc_dt);
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

pub async fn scrape_imperva_application_security_sitemap() -> Result<ScrapedEngineeringItems> {
    let cache_constant = imperva_cache_constant(IMPERVA_LEARN_APPLICATION_SECURITY_SITEMAP_URL);
    match local_storage::find_stored_item(&cache_constant).await {
        Some(i) => Ok(i),
        None => {
            let res =
                request_imperva_sitemap(IMPERVA_LEARN_APPLICATION_SECURITY_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = ImpervaSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(&cache_constant, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

pub async fn scrape_imperva_availability_sitemap() -> Result<ScrapedEngineeringItems> {
    let cache_constant = imperva_cache_constant(IMPERVA_LEARN_AVAILABILITY_SITEMAP_URL);
    match local_storage::find_stored_item(&cache_constant).await {
        Some(i) => Ok(i),
        None => {
            let res = request_imperva_sitemap(IMPERVA_LEARN_AVAILABILITY_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = ImpervaSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(&cache_constant, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

pub async fn scrape_imperva_data_security_sitemap() -> Result<ScrapedEngineeringItems> {
    let cache_constant = imperva_cache_constant(IMPERVA_LEARN_DATA_SECURITY_SITEMAP_URL);
    match local_storage::find_stored_item(&cache_constant).await {
        Some(i) => Ok(i),
        None => {
            let res = request_imperva_sitemap(IMPERVA_LEARN_DATA_SECURITY_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = ImpervaSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(&cache_constant, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

pub async fn scrape_imperva_ddos_sitemap() -> Result<ScrapedEngineeringItems> {
    let cache_constant = imperva_cache_constant(IMPERVA_LEARN_DDOS_SITEMAP_URL);
    match local_storage::find_stored_item(&cache_constant).await {
        Some(i) => Ok(i),
        None => {
            let res = request_imperva_sitemap(IMPERVA_LEARN_DDOS_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = ImpervaSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(&cache_constant, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

pub async fn scrape_imperva_performance_sitemap() -> Result<ScrapedEngineeringItems> {
    let cache_constant = imperva_cache_constant(IMPERVA_LEARN_PERFORMANCE_SITEMAP_URL);
    match local_storage::find_stored_item(&cache_constant).await {
        Some(i) => Ok(i),
        None => {
            let res = request_imperva_sitemap(IMPERVA_LEARN_PERFORMANCE_SITEMAP_URL).await?;
            let reader = Reader::from_str(&res);
            let handler = ImpervaSitemap::default();
            let items = parse_xml_with(reader, handler)?;
            let storage_key = StorageKey::new(&cache_constant, None, Some(10));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}
