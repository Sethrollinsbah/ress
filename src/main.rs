mod utils;
use crate::api::websocket_handler;
mod services;

mod models;
mod api;

use crate::utils::mail;
use models::AppState;
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
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
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
use crate::models::{get_redis_value, set_redis_value, check_redis};
#[tokio::main]
async fn main() {
    let manager = SqliteConnectionManager::file("data.db");
    let db_pool = Pool::new(manager).expect("Failed to create database pool");

    // Initialize table if needed
    let conn = db_pool.get().expect("Failed to get db connection");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        value TEXT
    )",
        [],
    )
    .expect("Failed to create table");
    // initialize tracing
    if !check_redis() {
        println!("Redis is not running. Please start Redis.");
        std::process::exit(1);
    }

    println!("Redis is running. Proceeding with application...");
    tracing_subscriber::fmt::init();

    // Create Redis client
    let redis_client =
        redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to Redis");
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
        .with_state(shared_state);

    println!("🚀 Server running on http://0.0.0.0:3043");
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3043").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
