// src/api/mod.rs

pub mod handler;
pub mod ws;

pub use handler::{run_lighthouse_handler};
pub use ws::{websocket_handler,read_file_contents,get_new_content, handle_socket};
