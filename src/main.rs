mod utils;
use crate::api::websocket_handler;
mod services;
mod api;
mod models;

use dotenv::dotenv;
use axum::{
    extract::{
        ws::WebSocketUpgrade,
        Json, State,
    },
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use models::AppState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use redis::Client;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;
use log::{info, error};

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
    let shared_state = Arc::new(AppState {
        redis_client,
    });
    // build our application with a route
    let app = Router::new()
        .route(
            "/lighthouse",
            axum::routing::get(api::run_lighthouse_handler),
        )
        .route("/ws", axum::routing::get(websocket_handler))
        .with_state(shared_state);
    println!
    ("{:?}", format!("🚀 Server running on http://{:?}:{:?}", &server_address, &port_number));
    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("0.0.0.0:3012")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
