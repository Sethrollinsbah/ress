// src/models/mod.rs

pub mod api;
pub mod app;
pub mod database;
pub mod kv;
pub mod lighthouse;
pub mod r#struct;
pub mod appointments;

pub use appointments::{Appointment, CreateAppointmentRequest, AppointmentResponse};
pub use api::{Params, ParamsRunLighthouse};
pub use app::AppState;
pub use database::Person;
pub use kv::{
    check_redis, /* Params,  */ get_redis_value, set_redis_value, update_cloudflare_kv,
    RedisInput, RedisParams, RedisResponse, RedisSetParams,
};
pub use lighthouse::{
    Audit, AverageReport, Categories, CategoriesStats, Category, ComprehensiveReport, Root,
    ScoreStats,
};
pub use r#struct::UserData;
