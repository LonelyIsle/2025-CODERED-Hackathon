use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub url: String,
    pub fetched_at: DateTime<Utc>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub body_text: String,
    pub content_type: Option<String>,
    pub http_status: i32,
    pub lang: Option<String>,
    pub content_hash: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}