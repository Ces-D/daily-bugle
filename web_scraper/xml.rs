use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use log::{error, info, trace};
use quick_xml::{Reader, events::Event};
use reqwest::header;
use std::io::Read;

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
            _ => {
            }
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
