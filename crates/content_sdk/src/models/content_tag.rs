use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentTag {
    #[serde(deserialize_with = "deserialize_option_string_or_int")]
    pub id: Option<i32>,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub content_id: i32,
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub tag_id: i32,
    #[serde(deserialize_with = "deserialize_option_datetime")]
    pub created_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentTagRequest {
    pub content_id: i32,
    pub tag_id: i32,
}

/// Deserialize DateTime from string, handling missing colon in timezone offset
/// Supabase sometimes returns "2026-04-14T11:28:41.088219+00" instead of "2026-04-14T11:28:41.088219+00:00"
fn deserialize_option_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::String(s) => {
            // Try parsing as RFC3339 first
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }

            // If that fails, try adding colon to timezone offset (e.g., "+00" -> "+00:00")
            if s.len() > 3 {
                let last_three = &s[s.len() - 3..];
                if last_three.starts_with('+') || last_three.starts_with('-') {
                    let normalized = format!("{}:00", s);
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
                        return Ok(Some(dt.with_timezone(&Utc)));
                    }
                }
            }

            // Try parsing with chrono's flexible parser
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }

            Err(serde::de::Error::custom("Invalid datetime format"))
        }
        _ => Err(serde::de::Error::custom(
            "Expected null or string for datetime",
        )),
    }
}
