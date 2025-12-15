pub mod news;
pub mod weather;

use anyhow::{Context, Result, bail};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, CreateChatCompletionRequest,
    },
};
use log::info;
use reqwest::header;
use serde::Deserialize;
use std::io::Read;

trait IntoUrl {
    fn into_url(self) -> url::Url;
}

pub async fn request_url<T: for<'a> Deserialize<'a>>(
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

pub fn system_message(text: &str) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(text.to_string()),
        ..Default::default()
    })
}

pub fn developer_message(text: String) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
        content: ChatCompletionRequestDeveloperMessageContent::Text(text),
        ..Default::default()
    })
}

pub async fn make_chat_completion_request(request: CreateChatCompletionRequest) -> Result<String> {
    let client = Client::new();
    let response = client
        .chat()
        .create(request)
        .await
        .with_context(|| "Failed to create chat completion")?;
    let message = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .with_context(|| "Failed to gather first message from response")?;
    Ok(message)
}
