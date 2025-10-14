use crate::constant::MDN_SITEMAP_URL;
use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use quick_xml::{Reader, events::Event};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Serialize, Deserialize, Default)]
pub struct Url {
    loc: String,
    lastmod: Option<NaiveDate>,
}

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

fn parse_xml_sitemap(mut reader: Reader<&[u8]>) -> Result<Vec<Url>> {
    let mut urls: Vec<Url> = Vec::new();
    let mut buf = Vec::new();

    let mut current_url: Option<Url> = None;
    let mut current_element = String::new();
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => bail!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"url" => {
                        current_url = Some(Url::default());
                    }
                    b"loc" | b"lastmod" => {
                        current_element = String::from_utf8_lossy(name.as_ref()).to_string();
                        current_text.clear();
                    }
                    _ => {}
                }
            }

            Ok(Event::End(e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"url" => {
                        if let Some(url) = current_url.take() {
                            urls.push(url);
                        }
                    }
                    b"loc" | b"lastmod" => {
                        if let Some(url) = &mut current_url {
                            match current_element.as_str() {
                                "loc" => url.loc = current_text.clone(),
                                "lastmod" => {
                                    if let Ok(dt) =
                                        NaiveDate::parse_from_str(&current_text, "%Y-%m-%d")
                                    {
                                        url.lastmod = Some(dt);
                                    }
                                }
                                _ => {}
                            }
                        }
                        current_element.clear();
                        current_text.clear();
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(e)) => {
                if !current_element.is_empty() {
                    current_text.push_str(&e.decode()?.into_owned());
                }
            }

            _ => {}
        }
        buf.clear();
    }
    Ok(urls)
}

pub async fn scrape_mdn_sitemap() -> Result<Vec<Url>> {
    let res = request_mdn_sitemap().await?;
    let reader = Reader::from_str(&res);
    parse_xml_sitemap(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let entries = parse_xml_sitemap(reader).expect("Failed to parse xml content");
        assert_eq!(entries.len(), 4);
        let first = entries.first().unwrap();
        assert_eq!(first.loc, "https://developer.mozilla.org/en-US/");
        assert!(first.lastmod.is_none());
        let last = entries.last().unwrap();
        assert_eq!(
            last.loc,
            "https://developer.mozilla.org/en-US/docs/Games/Techniques/2D_collision_detection"
        );
        assert_eq!(
            last.lastmod,
            Some(NaiveDate::parse_from_str("2025-07-11", "%Y-%m-%d").unwrap())
        )
    }
}
