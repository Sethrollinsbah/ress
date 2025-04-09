use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResponse {
    pub success: bool,
    pub site: String,
    #[serde(rename = "jobId", skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>, // Optional for cached responses
    #[serde(rename = "queuedFor", skip_serializing_if = "Option::is_none")]
    pub queued_for: Option<u64>, // Optional for cached responses
    #[serde(rename = "routesFound")]
    pub routes_found: usize,
    pub routes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cacheHit")]
    pub cache_hit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastCrawled")]
    pub last_crawled: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastCrawledDate")]
    pub last_crawled_date: Option<String>,
}
