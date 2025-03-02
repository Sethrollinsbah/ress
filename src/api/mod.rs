// src/api/mod.rs

pub mod handler;
pub mod ws;

pub use handler::run_lighthouse_handler;
pub use ws::{get_new_content, handle_socket, read_file_contents, websocket_handler};
