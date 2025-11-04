use crate::models::responses::{IngestResponse, ListResponse, StatusResponse};
use crate::services::download::download_book;
use crate::utils::file::{create_datalake_path, DATALAKE_PATH};
use axum::{extract::Path, http::StatusCode, response::Json};
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};
use tracing::error;
type DownloadedBooks = Arc<Mutex<HashSet<u32>>>;

pub async fn ingest_book(
    Path(book_id): Path<u32>,
    downloaded_books: axum::extract::State<DownloadedBooks>,
) -> Result<Json<IngestResponse>, StatusCode> {
    match download_book(book_id).await {
        Ok(path) => {
            downloaded_books.lock().unwrap().insert(book_id);
            Ok(Json(IngestResponse {
                book_id,
                status: "downloaded".to_string(),
                path,
            }))
        }
        Err(e) => {
            error!("Failed to download book {}: {}", book_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn check_status(Path(book_id): Path<u32>) -> Json<StatusResponse> {
    let datalake_path = create_datalake_path(book_id);
    let header_path = format!("{}/header_{}.txt", datalake_path, book_id);
    let body_path = format!("{}/body_{}.txt", datalake_path, book_id);

    let status = if std::path::Path::new(&header_path).exists()
        && std::path::Path::new(&body_path).exists()
    {
        "available"
    } else {
        "not_found"
    };

    Json(StatusResponse {
        book_id,
        status: status.to_string(),
    })
}

pub async fn list_books() -> Json<ListResponse> {
    let mut books = Vec::new();

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
                                                    books.push(book_id);
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

    books.sort();
    books.dedup();

    Json(ListResponse {
        count: books.len(),
        books,
    })
}
