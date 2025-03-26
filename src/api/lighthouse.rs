use crate::models::ParamsRunLighthouse;
use crate::services::run_lighthouse_process;
// use axum::extract::Query;

// use crate::models::{AppState, LighthouseResponse, LighthouseStatus, ParamsRunLighthouse};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use crate::AppState;
use chrono::{DateTime, Duration, Utc};
use redis::Commands;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::json;

const CACHE_DURATION_DAYS: i64 = 7; // Cache results for one week

pub async fn run_lighthouse_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ParamsRunLighthouse>,
) -> (StatusCode, Json<LighthouseResponse>) {
    println!("Started run lighthouse handler for domain: {}", params.domain);
    
    // Create a unique key for this URL
    let url_key = format!("lighthouse:url:{}", params.domain);
    let url_results_key = format!("lighthouse:results:{}", params.domain);
    
    // Try to get an existing Redis client connection
    let mut redis_conn = match state.redis_client.get_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LighthouseResponse {
                    status: LighthouseStatus::Error,
                    message: "Internal server error: Redis connection failed".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    expires_at: None,
                    result_url: None,
                }),
            );
        }
    };
    
    // Check if this URL is currently being processed
    let processing: Option<String> = redis_conn.get(&url_key).unwrap_or(None);
    
    if let Some(timestamp_str) = processing {
        // URL is currently being processed
        return (
            StatusCode::ACCEPTED,
            Json(LighthouseResponse {
                status: LighthouseStatus::Processing,
                message: "Lighthouse analysis already in progress".to_string(),
                timestamp: timestamp_str,
                expires_at: None,
                result_url: None,
            }),
        );
    }
    
    // Check if we have cached results
    let cached_results: Option<String> = redis_conn.get(&url_results_key).unwrap_or(None);
    
    if let Some(result_json) = cached_results {
        // Parse the cached result
        if let Ok(cached_response) = serde_json::from_str::<LighthouseResponse>(&result_json) {
            // Parse the timestamp
            // Fix for the DateTime issue:
if let Ok(timestamp) = DateTime::parse_from_rfc3339(&cached_response.timestamp) {
    // Convert the parsed timestamp to UTC
    let timestamp_utc = timestamp.with_timezone(&Utc);
    let now = Utc::now();
    
    // Check if the result is still valid (less than a week old)
    if now - timestamp_utc < Duration::days(CACHE_DURATION_DAYS) {
        return (StatusCode::OK, Json(cached_response));
    }
    // Otherwise, continue and run a new analysis
}
        }
    }
    
    // Set the URL as being processed with the current timestamp
    let current_time = Utc::now();
    let timestamp = current_time.to_rfc3339();
    
    // Store in Redis that we're processing this URL
    let _: () = redis_conn.set(&url_key, &timestamp).unwrap_or(());
    // Set an expiration on the processing status (2 hours)
    let _: () = redis_conn.expire(&url_key, 7200).unwrap_or(());
    
    // Spawn a task to run the lighthouse process
// For the first error (report_url issue):
// In your tokio::task::spawn block, update the run_lighthouse_process call:
// Before spawning the task, clone the timestamp
let timestamp_for_task = timestamp.clone();

tokio::task::spawn(async move {
    // Generate a hardcoded report URL if the actual function doesn't return one
    let report_url = match run_lighthouse_process(params.domain.clone(), params.email, params.name).await {
        Ok(_) => {
            // Generate a URL since the function doesn't return one
            format!("https://lighthouse-reports.example.com/report/{}", params.domain)
        },
        Err(err) => {
            // Handle the error case
            let error_response = LighthouseResponse {
                status: LighthouseStatus::Error,
                message: format!("Lighthouse analysis failed: {}", err),
                timestamp: timestamp_for_task.clone(),
                expires_at: None,
                result_url: None,
            };
            
            // Store the error result in Redis
            if let Ok(mut conn) = state.redis_client.get_connection() {
                if let Ok(json_result) = serde_json::to_string(&error_response) {
                    let _: () = conn.set(&url_results_key, json_result).unwrap_or(());
                    // Set expiration for the error results (1 day)
                    let _: () = conn.expire(&url_results_key, 60 * 60 * 24).unwrap_or(());
                }
                // Delete the processing status
                let _: () = conn.del(&url_key).unwrap_or(());
            }
            
            // Return early from the task
            return;
        }
    };
    
    // Create success response
    let expires_at = (current_time + Duration::days(CACHE_DURATION_DAYS)).to_rfc3339();
    let result = LighthouseResponse {
        status: LighthouseStatus::Completed,
        message: "Lighthouse analysis completed successfully".to_string(),
        timestamp: timestamp_for_task,
        expires_at: Some(expires_at),
        result_url: Some(report_url),
    };
    
    // Store the result in Redis
    if let Ok(mut conn) = state.redis_client.get_connection() {
        if let Ok(json_result) = serde_json::to_string(&result) {
            let _: () = conn.set(&url_results_key, json_result).unwrap_or(());
            
            // For the second error (expire issue):
            // Convert CACHE_DURATION_DAYS to i64 explicitly for the expire method
            let expire_seconds: i64 = 60 * 60 * 24 * CACHE_DURATION_DAYS;
            let _: () = conn.expire(&url_results_key, expire_seconds).unwrap_or(());
        }
        
        // Delete the processing status
        let _: () = conn.del(&url_key).unwrap_or(());
    }
});
    
    // Return immediate response that processing has started
    (
        StatusCode::ACCEPTED,
        Json(LighthouseResponse {
            status: LighthouseStatus::Started,
            message: "Lighthouse analysis started".to_string(),
            timestamp: timestamp.clone(),
            expires_at: None,
            result_url: None,
        }),
    )
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LighthouseResponse {
    pub status: LighthouseStatus,
    pub message: String,
    pub timestamp: String,
    pub expires_at: Option<String>,
    pub result_url: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LighthouseStatus {
    Started,
    Processing,
    Completed,
    Error,
}
