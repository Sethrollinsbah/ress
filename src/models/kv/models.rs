// src/models/redis/models.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RedisInput {
    pub db: Option<u8>,
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct RedisParams {
    pub db: Option<u8>,
    pub key: String,
}

#[derive(Deserialize)]
pub struct RedisSetParams {
    pub db: Option<u8>,
}

#[derive(Serialize)]
pub struct RedisResponse {
    pub key: String,
    pub value: Option<String>,
    pub status: String,
}
