use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use std::collections::HashSet;

fn tokenize_text(text: &str) -> HashSet<String> {
    let re = Regex::new(r"\b[a-zA-Z]+\b").unwrap();
    re.find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .filter(|word| word.len() > 2)
        .collect()
}

fn extract_metadata_from_header(header_content: &str) -> (String, String, String) {
    let title_re = Regex::new(r"(?i)title:\s*(.+)").unwrap();
    let author_re = Regex::new(r"(?i)author:\s*(.+)").unwrap();
    let lang_re = Regex::new(r"(?i)language:\s*(.+)").unwrap();

    let title = title_re.captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let author = author_re.captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let language = lang_re.captures(header_content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "en".to_string());

    (title, author, language)
}

fn benchmark_tokenize_text(c: &mut Criterion) {
    let sample_text = "This is a sample text for benchmarking tokenization performance. It contains various words that should be processed efficiently.";

    c.bench_function("tokenize_text_small", |b| {
        b.iter(|| tokenize_text(black_box(sample_text)))
    });
}

fn benchmark_tokenize_text_large(c: &mut Criterion) {
    let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ".repeat(1000);

    c.bench_function("tokenize_text_large", |b| {
        b.iter(|| tokenize_text(black_box(&large_text)))
    });
}

fn benchmark_extract_metadata(c: &mut Criterion) {
    let sample_header = r#"
Title: Pride and Prejudice
Author: Jane Austen
Language: English
Release Date: August 26, 2006 [EBook #1342]
"#;

    c.bench_function("extract_metadata", |b| {
        b.iter(|| extract_metadata_from_header(black_box(sample_header)))
    });
}

fn benchmark_full_processing(c: &mut Criterion) {
    let sample_header = r#"
Title: Test Book
Author: Test Author
Language: English
"#;
    let sample_body = "This is the body of the book. It contains many words that need to be tokenized and indexed for search purposes.".repeat(100);

    c.bench_function("full_processing", |b| {
        b.iter(|| {
            let _metadata = extract_metadata_from_header(black_box(sample_header));
            let _words = tokenize_text(black_box(&sample_body));
        })
    });
}

criterion_group!(
    benches,
    benchmark_tokenize_text,
    benchmark_tokenize_text_large,
    benchmark_extract_metadata,
    benchmark_full_processing
);
criterion_main!(benches);