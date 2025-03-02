use chrono::{DateTime, Utc};
use uuid::Uid;

#[derive(Debug)]
pub struct Session {
    pub user_id: Uuid,
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub expiration_time: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub is_authenticated: bool,
    pub roles: Vec<String>,
    pub language: String,
    pub last_active: DateTime<Utc>,
}
