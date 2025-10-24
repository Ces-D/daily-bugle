use crate::constant::{
    APRIL_EVENTS_URL, AUGUST_EVENTS_URL, DECEMBER_EVENTS_URL, FEBRUARY_EVENTS_URL,
    JANUARY_EVENTS_URL, JULY_EVENTS_URL, JUNE_EVENTS_URL, MARCH_EVENTS_URL, MAY_EVENTS_URL,
    NOVEMBER_EVENTS_URL, OCTOBER_EVENTS_URL, SEPTEMBER_EVENTS_URL, TIMEOUT_STORAGE_PREFIX,
    TODAY_EVENTS_URL, WEEK_EVENTS_URL, WEEKEND_EVENTS_URL,
};
use anyhow::{Result, anyhow, bail};
use chrono::{DateTime, Datelike};
use chrono_tz::{America::New_York, Tz};
use headless_chrome::{Browser, Element, LaunchOptions, Tab, browser::default_executable};
use local_storage::key::StorageKey;
use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Copy)]
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

fn current_month_events_url() -> &'static str {
    let now = chrono::Local::now();
    let month = now.month();

    match month {
        1 => JANUARY_EVENTS_URL,
        2 => FEBRUARY_EVENTS_URL,
        3 => MARCH_EVENTS_URL,
        4 => APRIL_EVENTS_URL,
        5 => MAY_EVENTS_URL,
        6 => JUNE_EVENTS_URL,
        7 => JULY_EVENTS_URL,
        8 => AUGUST_EVENTS_URL,
        9 => SEPTEMBER_EVENTS_URL,
        10 => OCTOBER_EVENTS_URL,
        11 => NOVEMBER_EVENTS_URL,
        12 => DECEMBER_EVENTS_URL,
        _ => JANUARY_EVENTS_URL, // fallback for invalid months
    }
}

fn scrape_article_time(tab: Arc<Tab>) -> Result<DateTime<Tz>> {
    let time_el = tab
        .wait_for_element("time._articleTime_1wpy4_12")
        .expect("Unable to create time selector");
    let date_time_string = time_el.get_attribute_value("datetime")?;
    if let Some(datetime_str) = date_time_string {
        let parsed_datetime = DateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M:%S%:z")?;
        let new_york = parsed_datetime.with_timezone(&New_York);
        log::trace!("Scraped article time: {}", &datetime_str);
        Ok(new_york)
    } else {
        bail!("Unable to find datetime attribute");
    }
}

fn scrape_article_content_title(element: Element<'_>) -> Result<(String, Option<String>)> {
    let href = element.get_attribute_value("href")?;
    let title = element.get_inner_text()?;
    log::trace!("Scraped article title: {}", &title);
    Ok((title, href.map(|t| t.to_owned())))
}

fn scrape_article_content_tags(elements: Vec<Element<'_>>) -> Result<Vec<String>> {
    let mut tags = Vec::<String>::new();
    for tag in elements {
        let label = tag.get_inner_text()?;
        tags.push(label);
    }
    log::trace!("Scraped article tags: {:?}", &tags);
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
    log::trace!("Scraped article summary: {:?}", &summary);
    Ok((summary, summary_links))
}

fn scrape_article_content(tab: Arc<Tab>) -> Result<Vec<ArticleContent>> {
    let content_els = tab
        .wait_for_elements("article.tile._article_osmln_1")
        .expect("Unable to create time selector");
    let mut article_contents = Vec::<ArticleContent>::new();
    for content in content_els {
        let title_el = match content.find_element("div._title_osmln_9 a") {
            Ok(el) => el,
            Err(_) => {
                log::error!("Unable to find content title");
                continue;
            }
        };
        let tags_el = match content.find_elements("div._tileTags_osmln_50 span") {
            Ok(el) => el,
            Err(e) => {
                log::error!("Unable to find content tags: {:?}", e);
                Vec::new()
            }
        };
        let summary_el = match content.find_elements("div._summaryContainer_osmln_364 p") {
            Ok(el) => el,
            Err(_) => {
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

fn timeout_variant_cache_constant(variant: ThingsToDoCycle) -> String {
    format!("{}-{}", TIMEOUT_STORAGE_PREFIX, &variant.to_string())
}
fn timeout_variants_storage_key(
    variant: ThingsToDoCycle,
    article_time: DateTime<Tz>,
) -> StorageKey {
    let constant = timeout_variant_cache_constant(variant);
    let expires_in = match variant {
        ThingsToDoCycle::Today => 1,
        ThingsToDoCycle::Week => 7,
        ThingsToDoCycle::Weekend => 3,
        ThingsToDoCycle::Month => 30,
    };
    StorageKey::new(&constant, Some(article_time.to_utc()), Some(expires_in))
}

pub async fn scrape_things_to_do(variant: ThingsToDoCycle) -> Result<ThingsToDo> {
    let cache_constant = timeout_variant_cache_constant(variant);
    let cached_todo: Option<ThingsToDo> = local_storage::find_stored_item(&cache_constant).await;

    if cached_todo.is_some() {
        Ok(cached_todo.unwrap())
    } else {
        let launch_options = LaunchOptions::default_builder()
            .idle_browser_timeout(core::time::Duration::from_secs(60))
            .path(Some(default_executable().map_err(|e| anyhow!(e))?))
            .build()?;
        let browser = Browser::new(launch_options)?;
        let tab = browser.new_tab()?;

        match variant {
            ThingsToDoCycle::Today => tab.navigate_to(TODAY_EVENTS_URL)?,
            ThingsToDoCycle::Week => tab.navigate_to(WEEK_EVENTS_URL)?,
            ThingsToDoCycle::Weekend => tab.navigate_to(WEEKEND_EVENTS_URL)?,
            ThingsToDoCycle::Month => tab.navigate_to(current_month_events_url())?,
        };

        let article_time = scrape_article_time(tab.clone())?;
        let recent_todo = match variant {
            ThingsToDoCycle::Today => {
                let article_contents = scrape_article_content(tab.clone())?;
                ThingsToDo {
                    written: article_time.to_rfc2822(),
                    article: article_contents,
                }
            }
            ThingsToDoCycle::Week => {
                let article_contents = scrape_article_content(tab.clone())?;
                ThingsToDo {
                    written: article_time.to_rfc2822(),
                    article: article_contents,
                }
            }
            ThingsToDoCycle::Weekend => {
                let article_contents = scrape_article_content(tab.clone())?;
                ThingsToDo {
                    written: article_time.to_rfc2822(),
                    article: article_contents,
                }
            }
            ThingsToDoCycle::Month => {
                let article_contents = scrape_article_content(tab.clone())?;
                ThingsToDo {
                    written: article_time.to_rfc2822(),
                    article: article_contents,
                }
            }
        };
        let storage_key = timeout_variants_storage_key(variant, article_time);
        local_storage::write_item_to_storage(storage_key, &recent_todo).await;
        Ok(recent_todo)
    }
}
