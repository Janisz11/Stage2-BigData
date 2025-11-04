use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookResult {
    pub book_id: u32,
    pub title: String,
    pub author: String,
    pub language: String,
    pub year: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub filters: HashMap<String, String>,
    pub count: usize,
    pub results: Vec<BookResult>,
}