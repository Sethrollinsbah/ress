mod utils;
use crate::api::websocket_handler;
mod services;

mod api;
mod models;

use dotenv::dotenv;
use crate::models::{check_redis, get_redis_value, set_redis_value};
use crate::utils::mail;
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
use models::AppState;
use notify::{Event, RecursiveMode, Watcher};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use redis::{AsyncCommands, Client, Commands};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::{path::Path, fs as stdfs, sync::Arc};
use tokio::sync::Mutex;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    net::TcpListener,
    sync::mpsc::channel,
};
use tracing_subscriber;
use log::{info, error};

use rusqlite::{Connection, Result};


#[tokio::main]
async fn main() {
    dotenv().ok();
    let server_address = std::env::var("SERVER_ADDRESS");
    let port_number = std::env::var("PORT_NUMBER");
    let manager = SqliteConnectionManager::file("data.db");
    let schema_file = "schema.sql";
    let db_pool = Pool::new(manager).expect("Failed to create database pool.");
    
    tracing_subscriber::fmt::init();
    let redis_url = match std::env::var("REDIS_URL") {
        Ok(url) => {
            info!("Using Redis URL from environment: {}", url);
            url
        }
        Err(_) => {
            error!("REDIS_URL environment variable is not set");
            std::process::exit(1);
        }
    };
    // Create Redis client
    let redis_client =
        redis::Client::open(format!("{}", &redis_url)).expect("Failed to connect to Redis");
        if !check_redis(&redis_url) {
        println!("Redis is not running. Please start Redis.");
        std::process::exit(1);
    }

    println!("Redis is running. Proceeding with application...");

    let shared_state = Arc::new(AppState {
        redis_client,
        db_pool,
    });

    // build our application with a route
    let app = Router::new()
        .route(
            "/lighthouse",
            axum::routing::get(api::run_lighthouse_handler),
        )
        .route("/ws", axum::routing::get(websocket_handler))
        .route("/kv", get(get_redis_value)) // Example Redis route
        .route("/kv", post(set_redis_value)) // POST route
        .route("/mail", post(mail::send_mail_handler))
        // .route("/appointments", post())
        .with_state(shared_state);

    println!("{:?}", format!("🚀 Server running on http://{:?}:{:?}", &server_address, &port_number));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3012")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
    };
    use axum_test_helper::TestClient;
    use redis::Commands;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_redis_kv_get_set() {
        let manager = SqliteConnectionManager::memory();
        let db_pool = Pool::new(manager).expect("Failed to create database pool");
        let redis_client =
            redis::Client::open(format!("redis://{}/", &server_address)).expect("Failed to connect to Redis");
        let shared_state = Arc::new(AppState {
            redis_client: redis_client.clone(),
        });

        let app = Router::new()
            .route("/kv", get(get_redis_value))
            .route("/kv", post(set_redis_value))
            .with_state(shared_state.clone());

        let client = TestClient::new(app);

        // Set a value
        let set_response = client
            .post("/kv")
            .header(header::CONTENT_TYPE, "application/json")
            .body(r#"{ "key": "test_key", "value": "test_value" }"#)
            .send()
            .await;

        assert_eq!(set_response.status(), StatusCode::OK);

        // Get the value
        let get_response = client.get("/kv?key=test_key").send().await;

        assert_eq!(get_response.status(), StatusCode::OK);
        let body = get_response.text().await;
        assert!(body.contains(r#""value":"test_value""#));
    }
}
