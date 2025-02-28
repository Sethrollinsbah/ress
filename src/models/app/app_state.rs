use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use redis;

#[derive(Clone)]
pub struct AppState {
    pub redis_client: redis::Client,
    pub db_pool: Pool<SqliteConnectionManager>,
}
