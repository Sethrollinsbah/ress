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

mod api;
mod kv;
mod lighthouse;
mod mail;
mod model;
mod ws;

#[tokio::main]
async fn main() {
    // initialize tracing
    if !kv::check_redis() {
        println!("Redis is not running. Please start Redis.");
        std::process::exit(1);
    }

    println!("Redis is running. Proceeding with application...");
    tracing_subscriber::fmt::init();

    // Create Redis client
    let redis_client =
        redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to Redis");
    let shared_state = Arc::new(model::AppState { redis_client });

    // build our application with a route
    let app = Router::new()
        .route(
            "/lighthouse",
            axum::routing::get(api::run_lighthouse_handler),
        )
        .route("/ws", axum::routing::get(ws::websocket_handler))
        .route("/kv", get(kv::get_redis_value)) // Example Redis route
        .route("/kv", post(kv::set_redis_value)) // POST route
        .route("/mail", post(mail::send_mail_handler))
        .with_state(shared_state);

    println!("🚀 Server running on http://0.0.0.0:3043");
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3043").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
