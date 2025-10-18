use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub url: String,
}

#[derive(Serialize)]
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
}