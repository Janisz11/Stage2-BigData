use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

type DownloadedBooks = Arc<Mutex<HashSet<u32>>>;

mod models;
mod routes;
mod services;
mod utils;

use routes::{
    health::health_check,
    ingest::{check_status, ingest_book, list_books},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("ingestion_service=info,tower_http=info")
        .init();

    let downloaded_books: DownloadedBooks = Arc::new(Mutex::new(HashSet::new()));

    let app = Router::new()
        .route("/status", get(health_check))
        .route("/ingest/:book_id", post(ingest_book))
        .route("/ingest/status/:book_id", get(check_status))
        .route("/ingest/list", get(list_books))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(downloaded_books);

    let port = std::env::var("PORT").unwrap_or_else(|_| "7001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Ingestion service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
