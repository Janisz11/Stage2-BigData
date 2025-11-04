use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct BookResult {
    book_id: u32,
    title: String,
    author: String,
    language: String,
    year: Option<u32>,
}

fn tokenize_query(query: &str) -> Vec<String> {
    query.to_lowercase()
         .split_whitespace()
         .filter(|word| word.len() > 2)
         .map(|word| word.to_string())
         .collect()
}

fn matches_query(book: &BookResult, query_words: &[String]) -> bool {
    let book_text = format!("{} {}", book.title.to_lowercase(), book.author.to_lowercase());

    query_words.iter().any(|word| {
        book_text.contains(word)
    })
}

fn create_sample_books() -> HashMap<u32, BookResult> {
    let mut books = HashMap::new();

    books.insert(1342, BookResult {
        book_id: 1342,
        title: "Pride and Prejudice".to_string(),
        author: "Jane Austen".to_string(),
        language: "en".to_string(),
        year: Some(1813),
    });

    books.insert(84, BookResult {
        book_id: 84,
        title: "Frankenstein".to_string(),
        author: "Mary Wollstonecraft Shelley".to_string(),
        language: "en".to_string(),
        year: Some(1818),
    });

    // Add more books for benchmarking
    for i in 1000..2000 {
        books.insert(i, BookResult {
            book_id: i,
            title: format!("Test Book {}", i),
            author: format!("Test Author {}", i % 50),
            language: "en".to_string(),
            year: Some(1800 + (i % 200)),
        });
    }

    books
}

fn benchmark_tokenize_query(c: &mut Criterion) {
    let query = "pride prejudice jane austen";

    c.bench_function("tokenize_query", |b| {
        b.iter(|| tokenize_query(black_box(query)))
    });
}

fn benchmark_matches_query(c: &mut Criterion) {
    let book = BookResult {
        book_id: 1342,
        title: "Pride and Prejudice".to_string(),
        author: "Jane Austen".to_string(),
        language: "en".to_string(),
        year: Some(1813),
    };
    let query_words = vec!["pride".to_string(), "prejudice".to_string()];

    c.bench_function("matches_query", |b| {
        b.iter(|| matches_query(black_box(&book), black_box(&query_words)))
    });
}

fn benchmark_search_small_dataset(c: &mut Criterion) {
    let books = create_sample_books();
    let query_words = vec!["test".to_string(), "book".to_string()];

    c.bench_function("search_small_dataset", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for book in books.values() {
                if matches_query(black_box(book), black_box(&query_words)) {
                    results.push(book.clone());
                }
            }
            results
        })
    });
}

fn benchmark_search_with_filters(c: &mut Criterion) {
    let books = create_sample_books();
    let query_words = vec!["test".to_string()];
    let author_filter = "Test Author 25".to_string();

    c.bench_function("search_with_filters", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for book in books.values() {
                if matches_query(black_box(book), black_box(&query_words)) {
                    if book.author.to_lowercase().contains(&author_filter.to_lowercase()) {
                        results.push(book.clone());
                    }
                }
            }
            results
        })
    });
}

criterion_group!(
    benches,
    benchmark_tokenize_query,
    benchmark_matches_query,
    benchmark_search_small_dataset,
    benchmark_search_with_filters
);
criterion_main!(benches);