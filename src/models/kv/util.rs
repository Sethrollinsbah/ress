use crate::models::{AppState, UserData, RedisResponse, RedisInput, RedisSetParams,  RedisParams};
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
use redis::{self, AsyncCommands,  Commands};
use reqwest::{Client, Error};
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
    State(state): State<Arc<AppState>>,
    Query(params): Query<RedisParams>,
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
                Json(RedisResponse {
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
            Json(RedisResponse {
                key,
                value: Some(value),
                status: "Success".to_string(),
            }),
        )
            .into_response(), // ✅ Convert the response properly
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(RedisResponse {
                key,
                value: None,
                status: "Key not found".to_string(),
            }),
        )
            .into_response(), // ✅ Convert the response properly
    }
}

pub async fn set_redis_value(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RedisSetParams>,
    Json(payload): Json<RedisInput>,
) -> impl IntoResponse {
    let db_number = params.db.unwrap_or(0); // Default to database 0 if not provided

    // Create a Redis connection
    let mut con = match state.redis_client.get_async_connection().await {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RedisResponse {
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
            Json(RedisResponse {
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
            Json(RedisResponse {
                key: payload.key,
                value: Some(payload.value),
                status: "Value stored to redis database".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RedisResponse {
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

pub async fn update_cloudflare_kv(domain: &str, mut email_list: Vec<String>) -> Result<(), anyhow::Error> {
    let client = Client::new();
    let namespace_id = "b40fac2149234730ae88f4bb8bbf3c78";
    let account_id = "0e9b5fad61935c0d6483962f4a522a89";
    let api_base = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        account_id, namespace_id, domain
    );

    let auth_email = "sethryanrollins@gmail.com";
    let auth_key = "295cf5944fd33c2f53a43dee2766cd1749ba6"; // Replace with env variable for security

    // Fetch existing record
    let existing_response = client
        .get(&api_base)
        .header("X-Auth-Email", auth_email)
        .header("X-Auth-Key", auth_key)
        .send()
        .await?;

    // let mut email_list = vec!["sethryanrollins@gmail.com".to_string()];

    if existing_response.status().is_success() {
        if let Ok(existing_data) = existing_response.json::<UserData>().await {
            email_list = existing_data.email;
            if !email_list.contains(&"sethryanrollins@gmail.com".to_string()) {
                email_list.push("sethryanrollins@gmail.com".to_string());
            }
        }
    }

    // Updated data
    let updated_data = UserData {
        email: email_list,
        name: "user".to_string(),
        status: 200,
    };
    // Create Redis client
    let redis_client =
        redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to Redis");

    // Call the helper function and handle the result
    match set_redis_value_helper(&redis_client, 0, domain.to_string(), serde_json::to_string(&updated_data).unwrap()).await {
        Ok(_) => {
            println!("Cloudflare KV updated successfully for domain: {}", domain);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to update KV: {}", e);
            Err(anyhow::Error::msg(e.to_string()))  // If `Error` has a constructor that accepts `String`
        }
    }
}
