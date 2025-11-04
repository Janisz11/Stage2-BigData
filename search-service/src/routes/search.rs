use crate::models::responses::{BookResult, SearchResponse};
use crate::models::storage::{BookMetadata, StorageBackend};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub year: Option<u32>,
}

type Backend = Arc<dyn StorageBackend + Send + Sync>;

fn tokenize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 2)
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|word| !word.is_empty())
        .collect()
}

// No longer needed - we get year directly from metadata

async fn get_book_ids_for_words(
    words: &[String],
    backend: &Backend,
) -> Result<HashSet<u32>, StatusCode> {
    if words.is_empty() {
        return Ok(HashSet::new());
    }

    let mut result_sets = Vec::new();

    for word in words {
        match backend.search_word(word).await {
            Ok(book_ids) => {
                if book_ids.is_empty() {
                    // If any word has no results, the intersection will be empty
                    return Ok(HashSet::new());
                }
                result_sets.push(book_ids);
            }
            Err(e) => {
                error!("Failed to search for word '{}': {}", word, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Find intersection of all sets (books that contain ALL words)
    if result_sets.is_empty() {
        return Ok(HashSet::new());
    }

    let mut intersection = result_sets[0].clone();
    for set in result_sets.iter().skip(1) {
        intersection = intersection.intersection(set).cloned().collect();
    }

    Ok(intersection)
}

async fn get_book_metadata_batch(
    book_ids: &HashSet<u32>,
    backend: &Backend,
) -> Vec<BookMetadata> {
    let mut metadata_list = Vec::new();

    for &book_id in book_ids {
        match backend.get_book_metadata(book_id).await {
            Ok(Some(metadata)) => metadata_list.push(metadata),
            Ok(None) => {
                error!("No metadata found for book {}", book_id);
            }
            Err(e) => {
                error!("Failed to get metadata for book {}: {}", book_id, e);
            }
        }
    }

    metadata_list
}

fn apply_filters(
    metadata_list: Vec<BookMetadata>,
    params: &SearchParams,
) -> Vec<BookMetadata> {
    metadata_list
        .into_iter()
        .filter(|book| {
            // Apply author filter
            if let Some(ref author_filter) = params.author {
                if !book
                    .author
                    .to_lowercase()
                    .contains(&author_filter.to_lowercase())
                {
                    return false;
                }
            }

            // Apply language filter
            if let Some(ref language_filter) = params.language {
                if book.language != *language_filter {
                    return false;
                }
            }

            // Apply year filter
            if let Some(year_filter) = params.year {
                if book.year != Some(year_filter) {
                    return false;
                }
            }

            true
        })
        .collect()
}

pub async fn search_books(
    Query(params): Query<SearchParams>,
    State(backend): State<Backend>,
) -> Result<Json<SearchResponse>, StatusCode> {
    info!("Search query: {:?}", params);

    // Tokenize the search query
    let query_words = tokenize_query(&params.q);

    if query_words.is_empty() {
        return Ok(Json(SearchResponse {
            query: params.q.clone(),
            filters: HashMap::new(),
            count: 0,
            results: Vec::new(),
        }));
    }

    // Find books that contain all the search words
    let book_ids = get_book_ids_for_words(&query_words, &backend).await?;

    if book_ids.is_empty() {
        return Ok(Json(SearchResponse {
            query: params.q.clone(),
            filters: build_filters_map(&params),
            count: 0,
            results: Vec::new(),
        }));
    }

    // Get metadata for all matching books
    let all_metadata = get_book_metadata_batch(&book_ids, &backend).await;

    // Apply filters
    let filtered_metadata = apply_filters(all_metadata, &params);

    // Convert to response format
    let mut results: Vec<BookResult> = filtered_metadata
        .into_iter()
        .map(|book| BookResult {
            book_id: book.book_id,
            title: book.title.clone(),
            author: book.author.clone(),
            language: book.language.clone(),
            year: book.year,
        })
        .collect();

    // Sort results by book_id for consistency
    results.sort_by_key(|book| book.book_id);

    let filters = build_filters_map(&params);

    Ok(Json(SearchResponse {
        query: params.q,
        filters,
        count: results.len(),
        results,
    }))
}

fn build_filters_map(params: &SearchParams) -> HashMap<String, String> {
    let mut filters = HashMap::new();

    if let Some(ref author) = params.author {
        filters.insert("author".to_string(), author.clone());
    }
    if let Some(ref language) = params.language {
        filters.insert("language".to_string(), language.clone());
    }
    if let Some(year) = params.year {
        filters.insert("year".to_string(), year.to_string());
    }

    filters
}