use redis::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

// Lighthouse response status
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LighthouseStatus {
    Started,
    Processing,
    Completed,
    Error,
}

impl fmt::Display for LighthouseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            LighthouseStatus::Started => "started",
            LighthouseStatus::Processing => "processing",
            LighthouseStatus::Completed => "completed",
            LighthouseStatus::Error => "error",
        };
        write!(f, "{}", status_str)
    }
}

// Response for the Lighthouse handler
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LighthouseResponse {
    pub status: LighthouseStatus,
    pub message: String,
    pub timestamp: String,
    pub expires_at: Option<String>,
    pub result_url: Option<String>,
}


