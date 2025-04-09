use crate::models::{AppState, WsParams};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::{IntoResponse, Response},
};
use futures::{SinkExt, StreamExt};
use sqlx::{postgres::PgListener, PgPool};
use tokio::sync::mpsc::channel;

pub async fn websocket_handler(
    Query(params): Query<WsParams>, // Extract query parameters
    ws: WebSocketUpgrade,           // WebSocket upgrade handler
    State(state): State<AppState>,  // Extract shared state (AppState with database pool)
) -> Response {
    // Access the query parameters (e.g., filename) here
    println!("Called with params: {:?}", &params.filename);

    // Open WebSocket connection and handle it
    ws.on_upgrade(move |socket| handle_socket(socket, state.pg_pool.clone())) // Pass the database pool to the handler
}

pub async fn handle_socket(socket: WebSocket, pool: PgPool) {
    let (mut sender, mut receiver) = socket.split();
    // Set up a channel to receive notifications about changes
    let (tx, mut rx) = channel::<String>(100);

    // Start a task to listen for database changes via PostgreSQL LISTEN/NOTIFY
    let listener_handle = tokio::spawn(async move {
        listen_for_changes(pool, tx).await;
    });

    // Handle incoming WebSocket messages (if needed)
    let socket_handle = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(Message::Text(text)) => {
                    println!("Received message: {}", text);
                    // You could handle incoming messages here if needed
                }
                _ => (),
            }
        }
    });

    // Listen for notifications about changes and send them to WebSocket clients
    let notify_handle = tokio::spawn(async move {
        while let Some(notification) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(notification.into())).await {
                println!("Error sending notification: {}", e);
            }
        }
    });

    // Await all tasks to complete
    tokio::select! {
        _ = listener_handle => println!("Database listener task completed"),
        _ = socket_handle => println!("Socket task completed"),
        _ = notify_handle => println!("Notification task completed"),
    }
}

// Listen to PostgreSQL database changes and send notifications to the channel
async fn listen_for_changes(pool: PgPool, tx: tokio::sync::mpsc::Sender<String>) {
    // Create a PgListener from the pool
    let mut listener = PgListener::connect_with(&pool).await.unwrap();

    // Execute the LISTEN SQL command to listen to changes in the node_configurations table
    listener.listen("node_config_changes").await.unwrap();

    println!("Listening for changes...");

    // Wait for notifications from PostgreSQL
    while let Ok(notification) = listener.recv().await {
        // Once a notification is received, send it to the WebSocket client
        let payload = notification.payload().to_string();
        if let Err(e) = tx
            .send(format!("Node configuration has changed: {}", payload))
            .await
        {
            println!("Error sending to channel: {}", e);
            break;
        }
    }
}
