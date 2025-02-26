use crate::model;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use notify::{Event, RecursiveMode, Watcher};
use redis::{AsyncCommands, Client, Commands};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    net::TcpListener,
    sync::mpsc::channel,
};
use tracing_subscriber;

pub async fn get_redis_value(
    State(state): State<Arc<model::AppState>>,
    Query(params): Query<model::RedisParams>,
) -> impl IntoResponse {
    // Get the key from the query parameters
    let key = params.key;
    let db_number = params.db.unwrap_or(0); // Default to database 0 if not provided

    // Create a Redis client and connect asynchronously
    let mut con = match state.redis_client.get_async_connection().await {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(model::RedisResponse {
                    key,
                    value: None,
                    status: "Failed to connect to Redis".to_string(),
                }),
            )
                .into_response(); // ✅ Ensure the response is converted properly
        }
    };

    // Switch to the selected Redis database using redis::cmd
    match redis::cmd("SELECT")
        .arg(db_number)
        .query_async::<()>(&mut con)
        .await
    {
        Ok(_) => (),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to select Redis database {}", db_number),
            )
                .into_response()
        }
    }

    // Try to get the value for the provided key from Redis
    let result: Result<String, redis::RedisError> = con.get(&key).await;

    match result {
        Ok(value) => (
            StatusCode::OK,
            Json(model::RedisResponse {
                key,
                value: Some(value),
                status: "Success".to_string(),
            }),
        )
            .into_response(), // ✅ Convert the response properly
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(model::RedisResponse {
                key,
                value: None,
                status: "Key not found".to_string(),
            }),
        )
            .into_response(), // ✅ Convert the response properly
    }
}

pub async fn set_redis_value(
    State(state): State<Arc<model::AppState>>,
    Query(params): Query<model::RedisSetParams>,
    Json(payload): Json<model::RedisInput>,
) -> impl IntoResponse {
    let db_number = params.db.unwrap_or(0); // Default to database 0 if not provided

    // Create a Redis connection
    let mut con = match state.redis_client.get_async_connection().await {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(model::RedisResponse {
                    key: "".to_string(),
                    value: None,
                    status: "Key not found".to_string(),
                }),
            )
                .into_response()
        }
    };

    // Select the database
    if let Err(_) = redis::cmd("SELECT")
        .arg(db_number)
        .query_async::<()>(&mut con)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(model::RedisResponse {
                key: payload.key,
                value: None,
                status: "Fauled to select redis database".to_string(),
            }),
        )
            .into_response();
    }

    // Set the key-value pair
    match con.set::<_, _, ()>(&payload.key, &payload.value).await {
        Ok(_) => (
            StatusCode::OK,
            Json(model::RedisResponse {
                key: payload.key,
                value: Some(payload.value),
                status: "Value stored to redis database".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(model::RedisResponse {
                key: "".to_string(),
                value: None,
                status: "Failed to store value in redis database".to_string(),
            }),
        )
            .into_response(),
    }
}

pub fn check_redis() -> bool {
    match redis::Client::open("redis://127.0.0.1:6379").and_then(|client| client.get_connection()) {
        Ok(mut con) => con.ping::<String>().is_ok(), // Specify `String` as the return type
        Err(_) => false,
    }
}

pub async fn set_redis_value_helper(
    redis_client: &redis::Client,
    db_number: u8,
    key: String,
    value: String,
) -> Result<(), String> {
    // Create a Redis connection
    let mut con = redis_client
        .get_async_connection()
        .await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    // Select the database
    redis::cmd("SELECT")
        .arg(db_number)
        .query_async::<()>(&mut con)
        .await
        .map_err(|e| format!("Failed to select Redis database: {}", e))?;

    // Set the key-value pair
    con.set::<_, _, ()>(&key, &value)
        .await
        .map_err(|e| format!("Failed to store value in Redis: {}", e))?;

    Ok(())
}
