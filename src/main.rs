use crate::api::websocket_handler;
mod api;
mod models;
use axum::Router;
use dotenv::dotenv;
use log::error;
use models::AppState;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let server_address = std::env::var("SERVER_ADDRESS");
    let port_number = std::env::var("PORT_NUMBER");
    tracing_subscriber::fmt::init();

    // Use the provided PostgreSQL connection URL
    let database_url = "postgres://root:mysecretpassword@localhost:5432/local";

    // Create SQLx PostgreSQL connection pool
    let pg_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to create PostgreSQL connection pool");

    // Test SQLx PostgreSQL connection
    match sqlx::query("SELECT 1").execute(&pg_pool).await {
        Ok(_) => println!("Successfully connected to PostgreSQL"),
        Err(e) => {
            error!("Failed to connect to PostgreSQL: {}", e);
            std::process::exit(1);
        }
    }

    // Create shared state with PostgreSQL pool
    let shared_state = Arc::new(AppState { pg_pool });

    // Build our application with routes
    let app = Router::new()
        .route("/ws", axum::routing::get(websocket_handler))
        .with_state(shared_state);

    println!(
        "{:?}",
        format!(
            "🚀 Server running on http://{:?}:{:?}",
            &server_address, &port_number
        )
    );

    // Run our app with hyper, listening globally on port
    let port = match port_number {
        Ok(port) => port,
        Err(_) => "3012".to_string(),
    };

    // Create a listener on IPv6 unspecified address with the port
    let listener = TcpListener::bind(format!("[::]:{}", port)).await.unwrap();

    // Log the address
    println!("🚀 Server running on http://[::]:{}", port);
    axum::serve(listener, app).await.unwrap();
}
