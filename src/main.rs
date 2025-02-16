use axum;
use axum::routing::post;
use axum::Router;
use tokio;

mod api;
mod lighthouse;
mod mail;
mod model;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", axum::routing::get(api::run_lighthouse_handler))
        .route("/mail", post(mail::send_mail_handler));

    println!("🚀 Server running on http://0.0.0.0:3043");
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3043").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
// async fn main() {
//     let app = api::create_router();
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//
//
//     if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
//         eprintln!("❌ Server error: {}", e);
//     }
// }
//
