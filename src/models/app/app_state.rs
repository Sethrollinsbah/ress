use redis;

#[derive(Clone)]
pub struct AppState {
    pub redis_client: redis::Client,
}
