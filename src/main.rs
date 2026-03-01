mod api;
mod db;
mod models;
mod dashboard;

use axum::{
    routing::{get, post},
    Router,
    response::Html,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use clap::Parser;

#[derive(Parser)]
#[command(name = "claude-quota", about = "Claude Code quota monitoring server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Path to SQLite database file
    #[arg(short, long, default_value = "claude-quota.db")]
    database: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let db = db::Database::new(&args.database)
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/api/report", post(api::report_handler))
        .route("/api/users", get(api::users_handler))
        .route("/api/usage", get(api::usage_handler))
        .route("/api/summary", get(api::summary_handler))
        .route("/api/hourly", get(api::hourly_handler))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let addr = format!("0.0.0.0:{}", args.port);
    println!("Claude Quota Monitor listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn dashboard_handler() -> Html<String> {
    Html(dashboard::dashboard_html())
}
