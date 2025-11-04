use crate::models::responses::{IndexResponse, IndexStatusResponse, RebuildResponse};
use crate::models::storage::StorageBackend;
use crate::services::indexing::process_book;
use crate::utils::file::DATALAKE_PATH;
use axum::{extract::Path, http::StatusCode, response::Json};
use chrono::Utc;
use std::fs;
use std::sync::Arc;
use tracing::{error, info, warn};

type Backend = Arc<dyn StorageBackend + Send + Sync>;

pub async fn index_book(
    Path(book_id): Path<u32>,
    axum::extract::State(backend): axum::extract::State<Backend>,
) -> Result<Json<IndexResponse>, StatusCode> {
    info!("Indexing book {}", book_id);

    match process_book(book_id, &backend).await {
        Ok(()) => Ok(Json(IndexResponse {
            book_id,
            status: "indexed".to_string(),
        })),
        Err(e) => {
            error!("Failed to index book {}: {}", book_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn rebuild_index(
    axum::extract::State(backend): axum::extract::State<Backend>,
) -> Result<Json<RebuildResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    info!("Starting index rebuild");

    let mut books_processed = 0;

    if let Ok(entries) = fs::read_dir(DATALAKE_PATH) {
        for date_entry in entries.flatten() {
            if date_entry
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                if let Ok(subdir_entries) = fs::read_dir(date_entry.path()) {
                    for subdir_entry in subdir_entries.flatten() {
                        if subdir_entry
                            .file_type()
                            .map(|ft| ft.is_dir())
                            .unwrap_or(false)
                        {
                            if let Ok(file_entries) = fs::read_dir(subdir_entry.path()) {
                                for file_entry in file_entries.flatten() {
                                    if let Some(filename) = file_entry.file_name().to_str() {
                                        if filename.starts_with("header_")
                                            && filename.ends_with(".txt")
                                        {
                                            if let Some(book_id_str) = filename
                                                .strip_prefix("header_")
                                                .and_then(|s| s.strip_suffix(".txt"))
                                            {
                                                if let Ok(book_id) = book_id_str.parse::<u32>() {
                                                    match process_book(book_id, &backend).await {
                                                        Ok(()) => {
                                                            books_processed += 1;
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                "Failed to index book {}: {}",
                                                                book_id, e
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let elapsed = start_time.elapsed();
    info!(
        "Index rebuild complete: {} books processed in {:?}",
        books_processed, elapsed
    );

    Ok(Json(RebuildResponse {
        books_processed,
        elapsed_time: format!("{:.2}s", elapsed.as_secs_f64()),
    }))
}

pub async fn get_index_status(
    axum::extract::State(backend): axum::extract::State<Backend>,
) -> Json<IndexStatusResponse> {
    let (book_count, word_count) = backend.get_stats().await.unwrap_or((0, 0));

    let index_size_mb = (book_count * 1000 + word_count * 100) as f64 / 1_000_000.0;

    Json(IndexStatusResponse {
        books_indexed: book_count,
        last_update: Utc::now().to_rfc3339(),
        index_size_mb,
    })
}
