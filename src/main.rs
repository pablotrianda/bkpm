use std::sync::Arc;
use axum::{
    Router,
    routing::get,
    response::{Html, IntoResponse},
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod db;
mod models;
mod scheduler;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "/data/bkpm.db".to_string());
    let db = Arc::new(db::Db::new(&db_path).expect("Failed to open database"));

    let backup_dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "/backups".to_string());
    let mut scheduler = scheduler::Scheduler::new();
    scheduler.start(db.clone(), backup_dir);

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .merge(api::router(db.clone()));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3450".to_string())
        .parse()
        .unwrap_or(3450);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let html_path = std::env::var("HTML_PATH").unwrap_or_else(|_| "/app/static/index.html".to_string());
    let content = std::fs::read_to_string(&html_path).unwrap_or_default();
    Html(content)
}
