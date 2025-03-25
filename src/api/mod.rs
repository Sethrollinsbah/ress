// src/api/mod.rs

pub mod lighthouse;
pub mod ws;

pub use lighthouse::run_lighthouse_handler;
pub use ws::{get_new_content, handle_socket, read_file_contents, websocket_handler};
