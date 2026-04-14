use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct Tag {
    #[serde(deserialize_with = "deserialize_option_string_or_int")]
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    #[serde(deserialize_with = "deserialize_option_string_or_int")]
    pub parent_id: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
}

/// Deserialize a value that can be either a string or a number into an i32
fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num
            .as_i64()
            .map(|n| n as i32)
            .ok_or_else(|| serde::de::Error::custom("Invalid number")),
        Value::String(s) => s
            .parse::<i32>()
            .map_err(|_| serde::de::Error::custom("Invalid integer string")),
        _ => Err(serde::de::Error::custom("Expected string or number")),
    }
}

/// Deserialize an optional value that can be either a string or a number into Option<i32>
fn deserialize_option_string_or_int<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::Number(num) => Ok(num.as_i64().map(|n| n as i32)),
        Value::String(s) => Ok(s.parse::<i32>().ok()),
        _ => Err(serde::de::Error::custom("Expected null, string, or number")),
    }
}

/// Request structure for creating/updating tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct TagRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
}

impl From<Tag> for TagRequest {
    fn from(tag: Tag) -> Self {
        TagRequest {
            id: tag.id,
            name: tag.name,
            slug: tag.slug,
            parent_id: tag.parent_id,
        }
    }
}
