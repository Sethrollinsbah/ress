mod utils;
use crate::api::websocket_handler;
mod api;
mod models;
mod services;

use axum::{
    extract::{ws::WebSocketUpgrade, Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use log::{error, info};
use models::AppState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use redis::Client;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let server_address = std::env::var("SERVER_ADDRESS");
    let port_number = std::env::var("PORT_NUMBER");

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
    if !models::check_redis(&redis_url) {
        println!("Redis is not running. Please start Redis.");
        std::process::exit(1);
    }
    println!("Redis is running. Proceeding with application...");
    let shared_state = Arc::new(AppState { redis_client });
    // build our application with a route
    let app = Router::new()
        .route(
            "/lighthouse",
            axum::routing::get(api::run_lighthouse_handler),
        )
        .route("/ws", axum::routing::get(websocket_handler))
        .with_state(shared_state);
    println!(
        "{:?}",
        format!(
            "🚀 Server running on http://{:?}:{:?}",
            &server_address, &port_number
        )
    );
    // run our app with hyper, listening globally on port 3000
let port = match port_number {
    Ok(port) => port,
    Err(_) => "3012".to_string(),
};

// Create a listener on IPv6 unspecified address with the port
let listener = TcpListener::bind(format!("[::]:{}", port)).await.unwrap();

// Log the address
println!(
    "🚀 Server running on http://[::]:{}",
    port
);

    axum::serve(listener, app).await.unwrap();
}
