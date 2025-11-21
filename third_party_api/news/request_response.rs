use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use strum_macros::{Display, EnumString, VariantNames};

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorCode {
    code: u32,
    message: String,
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Error(ErrorCode),
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Status::Ok => serializer.serialize_str("ok"),
            Status::Error(err) => err.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string 'ok' or an error object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Status, E>
            where
                E: de::Error,
            {
                if value == "ok" {
                    Ok(Status::Ok)
                } else {
                    Err(E::custom(format!("unexpected status: {}", value)))
                }
            }

            fn visit_map<M>(self, map: M) -> Result<Status, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                ErrorCode::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(Status::Error)
            }
        }

        deserializer.deserialize_any(StatusVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, Display, EnumString, VariantNames)]
pub enum Country {
    #[serde(rename = "us")]
    #[strum(serialize = "us")]
    USA,
    #[serde(rename = "mx")]
    #[strum(serialize = "mx")]
    Mexico,
}

#[derive(Debug, Deserialize, Serialize, Display, EnumString, VariantNames)]
pub enum Category {
    #[serde(rename = "business")]
    #[strum(serialize = "business")]
    Business,
    #[serde(rename = "entertainment")]
    #[strum(serialize = "entertainment")]
    Entertainment,
    #[serde(rename = "general")]
    #[strum(serialize = "general")]
    General,
    #[serde(rename = "health")]
    #[strum(serialize = "health")]
    Health,
    #[serde(rename = "science")]
    #[strum(serialize = "science")]
    Science,
    #[serde(rename = "sports")]
    #[strum(serialize = "sports")]
    Sports,
    #[serde(rename = "technology")]
    #[strum(serialize = "technology")]
    Technology,
}

#[derive(Debug, Deserialize, Serialize, Display, EnumString, VariantNames)]
pub enum Language {
    #[serde(rename = "en")]
    #[strum(serialize = "en")]
    English,
    #[serde(rename = "es")]
    #[strum(serialize = "es")]
    Spanish,
}

/// see - https://newsapi.org/docs/endpoints/sources
#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    id: String,
    name: String,
    description: String,
    url: String,
    category: Category,
    language: Language,
    country: Country,
}

/// see - https://newsapi.org/docs/endpoints/sources
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseSources {
    pub status: Status,
    pub sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleSource {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Article {
    source: SimpleSource,
    author: Option<String>,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    url_to_image: Option<String>,
    published_at: Option<String>,
    content: Option<String>,
}

/// see - https://newsapi.org/docs/endpoints/top-headlines
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ResponseTopHeadlines {
    status: Status,
    total_results: u32,
    pub articles: Vec<Article>,
}
