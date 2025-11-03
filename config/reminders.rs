use anyhow::Context;
use chrono::{DateTime, Datelike, Month, TimeZone, Utc};
use rrule::{Frequency, NWeekday, RRule, Tz, Unvalidated, Validated, Weekday};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportantDate {
    pub name: String,
    pub category: String,
    #[serde(
        serialize_with = "serialize_rrule_to_string",
        deserialize_with = "deserialize_rrule_from_str"
    )]
    pub recurrence: RRule<Validated>,
    pub tags: Vec<String>,
}

pub fn serialize_rrule_to_string<S>(
    rrule: &RRule<Validated>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&rrule.to_string())
}

pub fn deserialize_rrule_from_str<'de, D>(deserializer: D) -> Result<RRule<Validated>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let unvalidated = RRule::<Unvalidated>::from_str(&s).map_err(|e| {
        serde::de::Error::custom(format!("Failed to parse RRule string '{}': {}", s, e))
    })?;

    // Use start of current year as reference date for validation
    // This provides a sensible default for yearly recurring events like holidays
    unvalidated
        .validate(current_year())
        .map_err(|e| serde::de::Error::custom(format!("Failed to validate RRule: {}", e)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminders {
    pub dates: Vec<ImportantDate>,
}

fn current_year() -> DateTime<Tz> {
    let now = Utc::now();
    rrule::Tz::UTC
        .with_ymd_and_hms(now.year(), 1, 1, 0, 0, 0)
        .unwrap()
}

pub fn defaults() -> anyhow::Result<Reminders> {
    Ok(Reminders {
        dates: vec![
            ImportantDate {
            name: "New Year's Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::January])
                .by_month_day(vec![1])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for New Year's Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Martin Luther King Jr. Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::January])
                .by_weekday(vec![NWeekday::Nth(3, Weekday::Mon)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Martin Luther King Jr. Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Presidents Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::February])
                .by_weekday(vec![NWeekday::Nth(3, Weekday::Mon)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Presidents Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Valentine's Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::February])
                .by_month_day(vec![14])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Valentine's Day")?,
            tags: vec![
                "federal".to_string(),
                "romantic".to_string(),
                "holiday".to_string(),
            ],
        },
        ImportantDate {
            name: "Mother's Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::May])
                .by_weekday(vec![NWeekday::Nth(2, Weekday::Sun)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Mother's Day")?,
            tags: vec!["family".to_string(), "holiday".to_string()],
        },
        ImportantDate {
            name: "Memorial Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::May])
                .by_weekday(vec![NWeekday::Nth(-1, Weekday::Mon)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Memorial Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Father's Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::June])
                .by_weekday(vec![NWeekday::Nth(3, Weekday::Sun)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Father's Day")?,
            tags: vec!["family".to_string(), "holiday".to_string()],
        },
        ImportantDate {
            name: "Independence Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::July])
                .by_month_day(vec![4])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Independence Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Labor Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::September])
                .by_weekday(vec![NWeekday::Nth(1, Weekday::Mon)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Labor Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Columbus Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::October])
                .by_weekday(vec![NWeekday::Nth(2, Weekday::Mon)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Columbus Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Halloween".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::October])
                .by_month_day(vec![31])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Halloween")?,
            tags: vec!["cultural".to_string()],
        },
        ImportantDate {
            name: "Veterans Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::November])
                .by_month_day(vec![11])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Veterans Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Thanksgiving".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::November])
                .by_weekday(vec![NWeekday::Nth(4, Weekday::Thu)])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Thanksgiving")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Christmas Day".to_string(),
            category: "holiday".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::December])
                .by_month_day(vec![25])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Christmas Day")?,
            tags: vec!["federal".to_string()],
        },
        ImportantDate {
            name: "Spring Equinox".to_string(),
            category: "nature".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::March])
                .by_month_day(vec![21])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Spring Equinox")?,
            tags: vec![
                "seasonal".to_string(),
                "equinox".to_string(),
                "nature".to_string(),
            ],
        },
        ImportantDate {
            name: "Summer Solstice (Longest Day)".to_string(),
            category: "nature".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::June])
                .by_month_day(vec![21])
                .validate(current_year())
                .with_context(
                    || "Failed to validate recurrence for Summer Solstice (Longest Day)",
                )?,
            tags: vec![
                "seasonal".to_string(),
                "solstice".to_string(),
                "nature".to_string(),
                "longest_day".to_string(),
            ],
        },
        ImportantDate {
            name: "Fall Equinox".to_string(),
            category: "nature".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::September])
                .by_month_day(vec![22])
                .validate(current_year())
                .with_context(|| "Failed to validate recurrence for Fall Equinox")?,
            tags: vec![
                "seasonal".to_string(),
                "equinox".to_string(),
                "nature".to_string(),
            ],
        },
        ImportantDate {
            name: "Winter Solstice (Shortest Day)".to_string(),
            category: "nature".to_string(),
            recurrence: RRule::new(Frequency::Yearly)
                .by_month(&[Month::December])
                .by_month_day(vec![21])
                .validate(current_year())
                .with_context(
                    || "Failed to validate recurrence for Winter Solstice (Shortest Day)",
                )?,
            tags: vec![
                "seasonal".to_string(),
                "solstice".to_string(),
                "nature".to_string(),
                "shortest_day".to_string(),
            ],
        },
        ],
    })
}
