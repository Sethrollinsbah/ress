use sqlx::{Pool, Postgres};

pub struct AppState {
    pub pg_pool: Pool<Postgres>,
}
