use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexResponse {
    pub book_id: u32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RebuildResponse {
    pub books_processed: usize,
    pub elapsed_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStatusResponse {
    pub books_indexed: usize,
    pub last_update: String,
    pub index_size_mb: f64,
}
