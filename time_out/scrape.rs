use crate::{
    cache,
    constant::{
        APRIL_EVENTS_URL, AUGUST_EVENTS_URL, DECEMBER_EVENTS_URL, FEBRUARY_EVENTS_URL,
        JANUARY_EVENTS_URL, JULY_EVENTS_URL, JUNE_EVENTS_URL, MARCH_EVENTS_URL, MAY_EVENTS_URL,
        NOVEMBER_EVENTS_URL, OCTOBER_EVENTS_URL, SEPTEMBER_EVENTS_URL, TODAY_EVENTS_URL,
        WEEK_EVENTS_URL, WEEKEND_EVENTS_URL,
    },
};
use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Datelike};
use chrono_tz::{America::New_York, Tz};
use headless_chrome::{Browser, Element, LaunchOptions, Tab, browser::default_executable};
use log::{debug, error, info, trace};
use reqwest::{
    StatusCode,
    header::{ACCEPT, COOKIE, HeaderMap, HeaderValue},
};
use std::{fmt::Display, path::PathBuf, sync::Arc};

pub enum ThingsToDoCycle {
    Today,
    Week,
    Weekend,
    Month,
}

impl Display for ThingsToDoCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            ThingsToDoCycle::Today => "today",
            ThingsToDoCycle::Week => "week",
            ThingsToDoCycle::Weekend => "weekend",
            ThingsToDoCycle::Month => "month",
        };
        write!(f, "{}", out)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ArticleContent {
    title: String,
    tags: Vec<String>,
    content: String,
    links: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThingsToDo {
    written: String,
    article: Vec<ArticleContent>,
}

#[deprecated]
async fn request_timeout(url: &str) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("text/html").unwrap());
    headers.insert(COOKIE, HeaderValue::from_str("_TO_Canary=main; session_id=e31d8d4f-818c-4762-b0b8-36688f1c63db; _TO_Variance=eyJub25lIjp0cnVlfQ==; _TO_AB_Testing=97").unwrap());
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .with_context(|| format!("Failed to make request: {}", url))?;
    if res.status() == StatusCode::OK {
        res.text()
            .await
            .with_context(|| "Failed to read response text")
    } else {
        bail!(
            "Error requesting Timeout: {},{:?}",
            res.status(),
            res.headers()
        )
    }
}

fn current_month_events_url() -> &'static str {
    let now = chrono::Local::now();
    let month = now.month();
    if month == 1 {
        return JANUARY_EVENTS_URL;
    }
    if month == 2 {
        return FEBRUARY_EVENTS_URL;
    }
    if month == 3 {
        return MARCH_EVENTS_URL;
    }
    if month == 4 {
        return APRIL_EVENTS_URL;
    }
    if month == 5 {
        return MAY_EVENTS_URL;
    }
    if month == 6 {
        return JUNE_EVENTS_URL;
    }
    if month == 7 {
        return JULY_EVENTS_URL;
    }
    if month == 8 {
        return AUGUST_EVENTS_URL;
    }
    if month == 9 {
        return SEPTEMBER_EVENTS_URL;
    }
    if month == 10 {
        return OCTOBER_EVENTS_URL;
    }
    if month == 11 {
        return NOVEMBER_EVENTS_URL;
    }
    if month == 12 {
        return DECEMBER_EVENTS_URL;
    }
    return JANUARY_EVENTS_URL;
}

fn scrape_article_time(tab: Arc<Tab>) -> Result<DateTime<Tz>> {
    let time_el = tab
        .wait_for_element("time._articleTime_1wpy4_12")
        .expect("Unable to create time selector");
    let date_time_string = time_el.get_attribute_value("datetime")?;
    if let Some(datetime_str) = date_time_string {
        let parsed_datetime = DateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M:%S%:z")?;
        let new_york = parsed_datetime.with_timezone(&New_York);
        trace!("Scraped article time: {}", &datetime_str);
        Ok(new_york)
    } else {
        bail!("Unable to find datetime attribute");
    }
}

fn scrape_article_content_title(element: Element<'_>) -> Result<(String, Option<String>)> {
    let href = element.get_attribute_value("href")?;
    let title = element.get_inner_text()?;
    trace!("Scraped article title: {}", &title);
    Ok((title, href.map(|t| t.to_owned())))
}

fn scrape_article_content_tags(elements: Vec<Element<'_>>) -> Result<Vec<String>> {
    let mut tags = Vec::<String>::new();
    for tag in elements {
        let label = tag.get_inner_text()?;
        tags.push(label);
    }
    trace!("Scraped article tags: {:?}", &tags);
    Ok(tags)
}

fn scrape_article_content_summary(elements: Vec<Element<'_>>) -> Result<(String, Vec<String>)> {
    let mut summary = String::new();
    let mut summary_links = Vec::<String>::new();
    for content in elements {
        let anchors = content.find_elements("a")?;
        let inner_text = content.get_inner_text()?;
        for link in anchors {
            if let Some(href) = link.get_attribute_value("href")? {
                summary_links.push(href.to_owned());
            }
        }
        summary += &inner_text;
    }
    trace!("Scraped article summary: {:?}", &summary);
    Ok((summary, summary_links))
}

fn scrape_article_content(tab: Arc<Tab>) -> Result<Vec<ArticleContent>> {
    let content_els = tab
        .wait_for_elements("article.tile._article_osmln_1")
        .expect("Unable to create time selector");
    let mut article_contents = Vec::<ArticleContent>::new();
    for (index, content) in content_els.iter().enumerate() {
        let title_el = match content.find_element("div._title_osmln_9 a") {
            Ok(el) => el,
            Err(_) => {
                error!("Unable to find content title");
                continue;
            }
        };
        let tags_el = match content.find_elements("div._tileTags_osmln_50 span") {
            Ok(el) => el,
            Err(e) => {
                error!("Unable to find content tags: {:?}", e);
                Vec::new()
            }
        };
        let summary_el = match content.find_elements("div._summaryContainer_osmln_364 p") {
            Ok(el) => el,
            Err(e) => {
                bail!("Unable to find content summary");
            }
        };
        let (title, title_link) = scrape_article_content_title(title_el)?;
        let tags = scrape_article_content_tags(tags_el)?;
        let (summary, mut summary_links) = scrape_article_content_summary(summary_el)?;
        if let Some(title_link) = title_link {
            summary_links.push(title_link);
        }
        article_contents.push(ArticleContent {
            title,
            tags,
            content: summary,
            links: summary_links,
        });
    }

    Ok(article_contents)
}

pub async fn scrape_things_to_do(
    variant: ThingsToDoCycle,
    cache_path: PathBuf,
) -> Result<ThingsToDo> {
    let launch_options = LaunchOptions::default_builder()
        .idle_browser_timeout(core::time::Duration::from_secs(60))
        .path(Some(default_executable().map_err(|e| anyhow!(e))?))
        .build()?;
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;

    let things_to_do = match variant {
        ThingsToDoCycle::Today => {
            tab.navigate_to(TODAY_EVENTS_URL)?;
            let article_time = scrape_article_time(tab.clone())?;
            match cache::find_cached_item(ThingsToDoCycle::Today, article_time, &cache_path) {
                Some(item) => item,
                None => {
                    trace!("{} cache not found", ThingsToDoCycle::Today);
                    let article_contents = scrape_article_content(tab.clone())?;
                    let t = ThingsToDo {
                        written: article_time.to_rfc2822(),
                        article: article_contents,
                    };
                    cache::write_item_to_cache(
                        ThingsToDoCycle::Today,
                        article_time,
                        &t,
                        cache_path,
                    );
                    t
                }
            }
        }
        ThingsToDoCycle::Week => {
            tab.navigate_to(WEEK_EVENTS_URL)?;
            let article_time = scrape_article_time(tab.clone())?;
            match cache::find_cached_item(ThingsToDoCycle::Week, article_time, &cache_path) {
                Some(item) => item,
                None => {
                    trace!("{} cache not found", ThingsToDoCycle::Week);
                    let article_contents = scrape_article_content(tab.clone())?;
                    let t = ThingsToDo {
                        written: article_time.to_rfc2822(),
                        article: article_contents,
                    };
                    cache::write_item_to_cache(ThingsToDoCycle::Week, article_time, &t, cache_path);
                    t
                }
            }
        }
        ThingsToDoCycle::Weekend => {
            tab.navigate_to(WEEKEND_EVENTS_URL)?;
            let article_time = scrape_article_time(tab.clone())?;
            match cache::find_cached_item(ThingsToDoCycle::Weekend, article_time, &cache_path) {
                Some(item) => item,
                None => {
                    trace!("{} cache not found", ThingsToDoCycle::Weekend);
                    let article_contents = scrape_article_content(tab.clone())?;
                    let t = ThingsToDo {
                        written: article_time.to_rfc2822(),
                        article: article_contents,
                    };
                    cache::write_item_to_cache(
                        ThingsToDoCycle::Weekend,
                        article_time,
                        &t,
                        cache_path,
                    );
                    t
                }
            }
        }
        ThingsToDoCycle::Month => {
            tab.navigate_to(current_month_events_url())?;
            let article_time = scrape_article_time(tab.clone())?;
            match cache::find_cached_item(ThingsToDoCycle::Month, article_time, &cache_path) {
                Some(item) => item,
                None => {
                    trace!("{} cache not found", ThingsToDoCycle::Month);
                    let article_contents = scrape_article_content(tab.clone())?;
                    let t = ThingsToDo {
                        written: article_time.to_rfc2822(),
                        article: article_contents,
                    };
                    cache::write_item_to_cache(
                        ThingsToDoCycle::Month,
                        article_time,
                        &t,
                        cache_path,
                    );
                    t
                }
            }
        }
    };

    Ok(things_to_do)
}
