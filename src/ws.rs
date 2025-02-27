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
use std::{self,:wa
    path::Path, sync::Arc};
use tokio::sync::Mutex;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    net::TcpListener,
    sync::mpsc::channel,
};
use tracing_subscriber;

pub async fn websocket_handler(
    Query(params): Query<model::Params>,
    ws: WebSocketUpgrade,
) -> Response {
    let path = format!("{}tmp/reports/{}.txt", stf::env::current_dir, params.filename);
    // Check if the file exists
    if fs::metadata(&path).await.is_err() {
        // If the file doesn't exist, return a 404 Not Found response
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, path))
}

pub async fn read_file_contents(path: &str) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err("File does not exist".to_string());
    }

    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(_) => {
            if !Path::new(path).exists() {
                return Err("File was deleted during reading".to_string());
            }
            Ok(contents)
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

pub fn get_new_content(old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut new_text = String::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => new_text.push_str(change.to_string().as_str()),
            _ => {}
        }
    }

    new_text
}

pub async fn handle_socket(socket: WebSocket, path: String) {
    let (mut sender, mut receiver) = socket.split();

    // Store the last known content
    let last_content = Arc::new(Mutex::new(String::new()));

    // Initial file check and read
    match read_file_contents(&path).await {
        Ok(contents) => {
            if let Err(e) = sender.send(Message::Text(contents.clone().into())).await {
                println!("Error sending initial contents: {}", e);
                return;
            }
            // Store initial content
            *last_content.lock().await = contents;
        }
        Err(e) => {
            let _ = sender
                .send(Message::Text(format!("Error: {}", e).into()))
                .await;
            return;
        }
    }

    let (tx, rx) = channel(100);
    let (close_tx, mut close_rx) = channel(1);
    let last_content_clone = Arc::clone(&last_content);

    let mut watcher =
        notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                let _ = tx.blocking_send(event);
            }
            Err(e) => println!("Watch error: {:?}", e),
        })
        .unwrap();

    watcher
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .unwrap();

    let file_watcher_handle = tokio::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            match event.kind {
                notify::EventKind::Modify(_) => {
                    match read_file_contents(&path).await {
                        Ok(new_contents) => {
                            let mut last = last_content_clone.lock().await;
                            let diff = get_new_content(&last, &new_contents);

                            // Only send if there's new content
                            if !diff.is_empty() {
                                if let Err(e) = sender.send(Message::Text(diff.into())).await {
                                    println!("Error sending updated contents: {}", e);
                                    break;
                                }
                                // Update last known content
                                *last = new_contents.to_string();
                            }
                        }
                        Err(e) => {
                            if let Err(send_err) = sender
                                .send(Message::Text(format!("Error: {}", e).into()))
                                .await
                            {
                                println!("Error sending error message: {}", send_err);
                                break;
                            }
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    if let Err(e) = sender.send(Message::Text("File was deleted".into())).await {
                        println!("Error sending file deletion message: {}", e);
                    }
                    if let Err(e) = sender.send(Message::Close(None)).await {
                        println!("Error sending close frame: {}", e);
                    }
                    let _ = close_tx.send(()).await;
                    break;
                }
                _ => {}
            }
        }
    });

    let socket_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(Message::Text(text))) => println!("Received message: {}", text),
                        _ => (),
                    }
                }
                _ = close_rx.recv() => break
            }
        }
    });

    tokio::select! {
        _ = file_watcher_handle => println!("File watcher task completed"),
        _ = socket_handle => println!("Socket task completed"),
    }
}
