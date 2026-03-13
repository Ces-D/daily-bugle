use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use log::{error, info, trace};
use quick_xml::{Reader, events::Event};
use reqwest::header;
use std::io::Read;

/// A generic XML handler that recognises the most common sitemap and feed
/// tags and converts them into `ScrapedEngineeringItem`s.
///
/// Recognised item containers: `url`, `entry`, `item`
/// Recognised field tags:
///   url   ← `loc`, `id`, `link`
///   title ← `title`
///   summary ← `content`, `summary`, `description`
///   published ← `published`, `pubDate`
///   updated ← `lastmod`, `updated`
///
/// Dates are attempted as RFC-3339 first, then as `%Y-%m-%d`.
#[derive(Default)]
pub struct CommonXMLHandler {
    items: Vec<crate::ScrapedEngineeringItem>,
    current_item: Option<crate::ScrapedEngineeringItem>,
    current_element: String,
    current_text: String,
}

impl CommonXMLHandler {
    fn is_item_tag(name: &[u8]) -> bool {
        matches!(name, b"url" | b"entry" | b"item")
    }

    fn is_field_tag(name: &[u8]) -> bool {
        matches!(
            name,
            b"loc"
                | b"id"
                | b"link"
                | b"title"
                | b"content"
                | b"summary"
                | b"description"
                | b"published"
                | b"pubDate"
                | b"lastmod"
                | b"updated"
        )
    }

    fn parse_datetime(text: &str) -> Option<DateTime<Utc>> {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(text) {
            return Some(dt.to_utc());
        }
        if let Ok(d) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
            return Some(naive_date_to_utc(d));
        }
        None
    }

    fn apply_field(&mut self) {
        if let Some(item) = &mut self.current_item {
            match self.current_element.as_str() {
                "loc" | "id" | "link" => item.url = self.current_text.clone(),
                "title" => item.title = self.current_text.clone(),
                "content" | "summary" | "description" => {
                    item.summary = Some(self.current_text.clone())
                }
                "published" | "pubDate" => {
                    item.published = Self::parse_datetime(&self.current_text);
                }
                "lastmod" | "updated" => {
                    item.updated = Self::parse_datetime(&self.current_text);
                }
                _ => {}
            }
        }
    }
}

impl XMLHandler<Vec<crate::ScrapedEngineeringItem>> for CommonXMLHandler {
    fn start(&mut self, name: &[u8]) -> Result<()> {
        if Self::is_item_tag(name) {
            self.current_item = Some(crate::ScrapedEngineeringItem::default());
        } else if Self::is_field_tag(name) {
            self.current_element = String::from_utf8_lossy(name).to_string();
            self.current_text.clear();
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
        if Self::is_item_tag(name) {
            if let Some(item) = self.current_item.take() {
                self.items.push(item);
            }
        } else if Self::is_field_tag(name) {
            self.apply_field();
            self.current_element.clear();
            self.current_text.clear();
        }
        Ok(())
    }

    fn items(self) -> Vec<crate::ScrapedEngineeringItem> {
        self.items
    }
}

pub fn naive_date_to_utc(date: NaiveDate) -> DateTime<Utc> {
    let naive_dt = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let dt = New_York.from_local_datetime(&naive_dt).unwrap();
    dt.to_utc()
}

pub trait XMLHandler<T> {
    /// Called for `<tag ...>`
    fn start(&mut self, name: &[u8]) -> Result<()>;
    /// Called for text between start & end (already decoded to UTF-8)
    fn text(&mut self, txt: &str) -> Result<()>;
    /// Called for `</tag>`
    fn end(&mut self, name: &[u8]) -> Result<()>;
    fn items(self) -> T;
}

/// Generic parse loop (reusable for any XML)
pub fn parse_xml_with<H, T>(reader: Reader<&[u8]>, handler: H) -> Result<T>
where
    H: XMLHandler<T>,
{
    let mut buf = Vec::new();
    let mut reader = reader;
    let mut handler = handler;
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                error!("Error while parsing xml: {:?}", e);
                bail!("Error at position {}: {:?}", reader.error_position(), e)
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                trace!(
                    "Event::Start: {}",
                    String::from_utf8_lossy(e.name().as_ref())
                );
                handler.start(e.name().as_ref())?
            }
            Ok(Event::Text(e)) => {
                trace!("Event::Text: {}", String::from_utf8_lossy(e.as_ref()));
                handler.text(&e.decode()?.into_owned())?
            }
            Ok(Event::End(e)) => {
                trace!("Event::End: {}", String::from_utf8_lossy(e.name().as_ref()));
                handler.end(e.name().as_ref())?
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(handler.items())
}

pub async fn request_url_document_text(
    url: &str,
    headers: Option<header::HeaderMap>,
) -> Result<String> {
    info!("Requesting url: {}", url);
    let headers = headers.unwrap_or_default();
    let accepted_encoding = headers.get(header::ACCEPT_ENCODING);
    let builder = reqwest::ClientBuilder::new().default_headers(headers.clone());
    let client = builder
        .build()
        .with_context(|| "Unable to create request client")?;
    let res = client.get(url).send().await?;
    if res.status() != reqwest::StatusCode::OK {
        bail!("Failed request to {} - {}", url, res.status());
    } else {
        if let Some(encoding) = accepted_encoding {
            match encoding.to_str().unwrap_or_default() {
                "gzip, deflate" => {
                    let bytes = res
                        .bytes()
                        .await
                        .with_context(|| "Failed to decode response body")?;
                    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
                    let mut xml = String::new();
                    decoder.read_to_string(&mut xml)?;
                    Ok(xml)
                }
                _ => bail!("Unsupported encoding: {:?}", encoding),
            }
        } else {
            let xml = res.text().await?;
            Ok(xml)
        }
    }
}
