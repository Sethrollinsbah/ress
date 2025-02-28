use crate::services::run_lighthouse_process;
use axum::extract::Query;
use crate::models::ParamsRunLighthouse;

pub async fn run_lighthouse_handler(Query(params): Query<ParamsRunLighthouse>) -> &'static str {
    println!("Started run lighthouse handler");
    tokio::task::spawn(async move {
        let _ = run_lighthouse_process(params.domain, params.email, params.name).await;
    });

    "OK"
}
