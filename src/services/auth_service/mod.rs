// src/services/auth_service.rs
pub mod register;
pub mod verify;

pub use register::{register_user};
pub use verify::{verify_user_credentials}
