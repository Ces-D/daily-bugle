pub mod news;
pub mod weather;

use anyhow::{Context, Result, bail};
use log::info;
use reqwest::header;
use serde::Deserialize;
use std::io::Read;

trait IntoUrl {
    fn into_url(self) -> url::Url;
}

async fn request_url<T: for<'a> Deserialize<'a>>(
    url: &str,
    headers: Option<header::HeaderMap>,
) -> Result<T> {
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
                "deflate, gzip, br" => {
                    let bytes = res
                        .bytes()
                        .await
                        .with_context(|| "Failed to decode response body")?;
                    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
                    let mut decrypted = String::new();
                    decoder.read_to_string(&mut decrypted)?;
                    serde_json::from_str::<T>(&decrypted)
                        .with_context(|| format!("Failed to deserialize response of: {}", url))
                }
                _ => bail!("Unsupported encoding: {:?}", encoding),
            }
        } else {
            res.json::<T>()
                .await
                .with_context(|| format!("Failed to deserialize json response of: {}", url))
        }
    }
}
