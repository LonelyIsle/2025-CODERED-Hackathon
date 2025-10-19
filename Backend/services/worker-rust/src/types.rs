use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub url: String,
    pub fetched_at: DateTime<Utc>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub body_text: String,

    pub content_type: Option<String>,
    pub http_status: i32,

    // Extra metadata
    pub content_hash: Option<String>,
    pub etag: Option<String>,
    pub lang: Option<String>,

    // Added to align with DB schema
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub status: String,
}