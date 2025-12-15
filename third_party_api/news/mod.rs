pub mod request_response;

use crate::{IntoUrl, request_url};
use anyhow::{Context, Result, bail};
use local_storage::key::StorageKey;
use log::trace;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::str::FromStr;

const NEWS_URL: &str = "https://newsapi.org/v2/";
const NEWS_TOP_HEADLINES_STORAGE_CONSTANT: &str = "news_top_headlines";

#[derive(Debug, Default)]
pub struct HeadlineSourceUrl {
    pub api_key: String,
    pub country: Option<request_response::Country>,
    pub language: Option<request_response::Language>,
    pub category: Option<request_response::Category>,
}

impl IntoUrl for HeadlineSourceUrl {
    fn into_url(self) -> url::Url {
        let mut url = url::Url::from_str(NEWS_URL).unwrap();
        url = url
            .join("top-headlines/sources")
            .expect("Failed to join news url");
        url.query_pairs_mut().append_pair("apiKey", &self.api_key);
        if let Some(country) = self.country {
            url.query_pairs_mut()
                .append_pair("country", country.to_string().as_str());
        }
        if let Some(language) = self.language {
            url.query_pairs_mut()
                .append_pair("language", language.to_string().as_str());
        }
        if let Some(category) = self.category {
            url.query_pairs_mut()
                .append_pair("category", category.to_string().as_str());
        }
        trace!("News Headline Sources url: {:?}", url.to_string());
        url
    }
}

pub async fn top_headline_sources(
    url: HeadlineSourceUrl,
) -> Result<request_response::ResponseSources> {
    let client = reqwest::Client::new();
    let res = client
        .get(url.into_url())
        .send()
        .await
        .with_context(|| "Failed to get top headline sources")?;
    if res.status() == reqwest::StatusCode::OK {
        match res.json::<request_response::ResponseSources>().await {
            Ok(body) => Ok(body),
            Err(e) => bail!("Failed to parse request body: {}", e),
        }
    } else {
        bail!(
            "Error requesting top headline sources: Status {}",
            res.status()
        )
    }
}

#[derive(Debug, Default)]
pub struct TopHeadlinesUrl {
    pub api_key: String,
    pub country: Option<request_response::Country>,
    pub category: Option<request_response::Category>,
    pub sources: Option<Vec<String>>,
    pub query: Option<String>,
    pub page_size: Option<u32>,
    pub page: Option<u32>,
}

fn get_random_agent() -> String {
    let user_agents= vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:89.0) Gecko/20100101 Firefox/89.0".to_string(),
    ];
    let random_index = rand::random_range(..user_agents.len());
    user_agents.get(random_index).unwrap().clone()
}

impl IntoUrl for TopHeadlinesUrl {
    fn into_url(self) -> url::Url {
        let mut url = url::Url::from_str(NEWS_URL).unwrap();
        url = url.join("top-headlines").expect("Failed to join news url");
        url.query_pairs_mut().append_pair("apiKey", &self.api_key);
        if let Some(country) = self.country {
            url.query_pairs_mut()
                .append_pair("country", country.to_string().as_str());
        }
        if let Some(category) = &self.category {
            url.query_pairs_mut()
                .append_pair("category", category.to_string().as_str());
        }
        if let Some(sources) = self.sources {
            if self.category.is_some() {
                panic!("Cannot specify both category and sources");
            }
            url.query_pairs_mut()
                .append_pair("sources", sources.join(",").as_str());
        }
        if let Some(query) = self.query {
            url.query_pairs_mut().append_pair("q", query.as_str());
        }
        if let Some(page_size) = self.page_size {
            url.query_pairs_mut()
                .append_pair("pageSize", page_size.to_string().as_str());
        }
        if let Some(page) = self.page {
            url.query_pairs_mut()
                .append_pair("page", page.to_string().as_str());
        }
        trace!("News Top Headlines url: {:?}", url.to_string());
        url
    }
}

pub async fn top_headlines(url: TopHeadlinesUrl) -> Result<request_response::ResponseTopHeadlines> {
    match local_storage::find_stored_item(NEWS_TOP_HEADLINES_STORAGE_CONSTANT).await {
        Some(body) => Ok(body),
        None => {
            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(&get_random_agent()).unwrap(),
            );
            let res = request_url::<request_response::ResponseTopHeadlines>(
                url.into_url().as_str(),
                Some(headers),
            )
            .await?;
            let storage_key = StorageKey::new(NEWS_TOP_HEADLINES_STORAGE_CONSTANT, None, Some(2));
            local_storage::write_item_to_storage(storage_key, &res).await;
            Ok(res)
        }
    }
}
