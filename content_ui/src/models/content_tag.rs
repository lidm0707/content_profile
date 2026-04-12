use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentTag {
    pub id: Option<i32>,
    pub content_id: i32,
    pub tag_id: i32,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Props)]
pub struct ContentTagRequest {
    pub content_id: i32,
    pub tag_id: i32,
}
