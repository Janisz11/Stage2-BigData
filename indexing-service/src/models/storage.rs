use async_trait::async_trait;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashSet;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("PostgreSQL error: {0}")]
    Postgres(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Connection error: {0}")]
    Connection(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub book_id: u32,
    pub title: String,
    pub author: String,
    pub language: String,
    pub year: Option<u32>,
    pub word_count: usize,
    pub unique_words: usize,
}

#[async_trait]
pub trait StorageBackend {
    async fn store_book_metadata(&self, metadata: &BookMetadata) -> Result<(), StorageError>;
    async fn get_book_metadata(&self, book_id: u32) -> Result<Option<BookMetadata>, StorageError>;
    async fn is_book_indexed(&self, book_id: u32) -> Result<bool, StorageError>;
    async fn get_indexed_books(&self) -> Result<HashSet<u32>, StorageError>;
    async fn add_word_to_index(&self, word: &str, book_id: u32) -> Result<(), StorageError>;
    async fn search_word(&self, word: &str) -> Result<HashSet<u32>, StorageError>;
    async fn get_stats(&self) -> Result<(usize, usize), StorageError>; // (total_books, unique_words)
    async fn test_connection(&self) -> Result<(), StorageError>;
}

pub struct RedisBackend {
    client: redis::Client,
}

impl RedisBackend {
    pub fn new(redis_url: &str) -> Result<Self, StorageError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, StorageError> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }
}

#[async_trait]
impl StorageBackend for RedisBackend {
    async fn store_book_metadata(&self, metadata: &BookMetadata) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;

        let key = format!("book:{}:metadata", metadata.book_id);
        let value = serde_json::to_string(metadata)?;

        conn.set::<_, _, ()>(&key, &value).await?;
        conn.incr::<_, _, ()>("stats:total_books", 1).await?;

        Ok(())
    }

    async fn get_book_metadata(&self, book_id: u32) -> Result<Option<BookMetadata>, StorageError> {
        let mut conn = self.get_connection().await?;

        let key = format!("book:{}:metadata", book_id);
        let value: Option<String> = conn.get(&key).await?;

        match value {
            Some(json_str) => {
                let metadata: BookMetadata = serde_json::from_str(&json_str)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    async fn is_book_indexed(&self, book_id: u32) -> Result<bool, StorageError> {
        let mut conn = self.get_connection().await?;

        let key = format!("book:{}:metadata", book_id);
        let exists: bool = conn.exists(&key).await?;

        Ok(exists)
    }

    async fn get_indexed_books(&self) -> Result<HashSet<u32>, StorageError> {
        let mut conn = self.get_connection().await?;

        let pattern = "book:*:metadata";
        let keys: Vec<String> = conn.keys(pattern).await?;

        let mut book_ids = HashSet::new();
        for key in keys {
            if let Some(book_id_str) = key
                .strip_prefix("book:")
                .and_then(|s| s.strip_suffix(":metadata"))
            {
                if let Ok(book_id) = book_id_str.parse::<u32>() {
                    book_ids.insert(book_id);
                }
            }
        }

        Ok(book_ids)
    }

    async fn add_word_to_index(&self, word: &str, book_id: u32) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;

        let word_key = format!("word:{}", word);
        conn.sadd::<_, _, ()>(&word_key, book_id).await?;
        conn.sadd::<_, _, ()>("stats:all_words", word).await?;

        Ok(())
    }

    async fn search_word(&self, word: &str) -> Result<HashSet<u32>, StorageError> {
        let mut conn = self.get_connection().await?;

        let word_key = format!("word:{}", word);
        let book_ids: Vec<u32> = conn.smembers(&word_key).await?;

        Ok(book_ids.into_iter().collect())
    }

    async fn get_stats(&self) -> Result<(usize, usize), StorageError> {
        let mut conn = self.get_connection().await?;

        let total_books: Option<usize> = conn.get("stats:total_books").await?;
        let unique_words: usize = conn.scard("stats:all_words").await?;

        Ok((total_books.unwrap_or(0), unique_words))
    }

    async fn test_connection(&self) -> Result<(), StorageError> {
        let mut conn = self.get_connection().await?;
        let _: Option<String> = conn.get("__connection_test__").await?;
        Ok(())
    }
}

pub struct PostgresBackend {
    pool: PgPool,
}

impl PostgresBackend {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url).await?;

        // Initialize tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS books (
                book_id INTEGER PRIMARY KEY,
                title TEXT,
                author TEXT,
                language VARCHAR(10),
                year INTEGER,
                word_count INTEGER,
                unique_words INTEGER,
                indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS word_index (
                word VARCHAR,
                book_id INTEGER,
                PRIMARY KEY (word, book_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_word_index_word ON word_index(word)
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageBackend for PostgresBackend {
    async fn store_book_metadata(&self, metadata: &BookMetadata) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO books (book_id, title, author, language, year, word_count, unique_words)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (book_id) DO UPDATE SET
                title = EXCLUDED.title,
                author = EXCLUDED.author,
                language = EXCLUDED.language,
                year = EXCLUDED.year,
                word_count = EXCLUDED.word_count,
                unique_words = EXCLUDED.unique_words,
                indexed_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(metadata.book_id as i32)
        .bind(&metadata.title)
        .bind(&metadata.author)
        .bind(&metadata.language)
        .bind(metadata.year.map(|y| y as i32))
        .bind(metadata.word_count as i32)
        .bind(metadata.unique_words as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_book_metadata(&self, book_id: u32) -> Result<Option<BookMetadata>, StorageError> {
        let row = sqlx::query(
            "SELECT book_id, title, author, language, year, word_count, unique_words FROM books WHERE book_id = $1"
        )
        .bind(book_id as i32)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let metadata = BookMetadata {
                    book_id: row.get::<i32, _>("book_id") as u32,
                    title: row.get("title"),
                    author: row.get("author"),
                    language: row.get("language"),
                    year: row.get::<Option<i32>, _>("year").map(|y| y as u32),
                    word_count: row.get::<i32, _>("word_count") as usize,
                    unique_words: row.get::<i32, _>("unique_words") as usize,
                };
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    async fn is_book_indexed(&self, book_id: u32) -> Result<bool, StorageError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM books WHERE book_id = $1)")
            .bind(book_id as i32)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get(0))
    }

    async fn get_indexed_books(&self) -> Result<HashSet<u32>, StorageError> {
        let rows = sqlx::query("SELECT book_id FROM books")
            .fetch_all(&self.pool)
            .await?;

        let book_ids = rows
            .into_iter()
            .map(|row| row.get::<i32, _>("book_id") as u32)
            .collect();

        Ok(book_ids)
    }

    async fn add_word_to_index(&self, word: &str, book_id: u32) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO word_index (word, book_id) VALUES ($1, $2) ON CONFLICT (word, book_id) DO NOTHING"
        )
        .bind(word)
        .bind(book_id as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn search_word(&self, word: &str) -> Result<HashSet<u32>, StorageError> {
        let rows = sqlx::query("SELECT book_id FROM word_index WHERE word = $1")
            .bind(word)
            .fetch_all(&self.pool)
            .await?;

        let book_ids = rows
            .into_iter()
            .map(|row| row.get::<i32, _>("book_id") as u32)
            .collect();

        Ok(book_ids)
    }

    async fn get_stats(&self) -> Result<(usize, usize), StorageError> {
        let total_books = sqlx::query("SELECT COUNT(*) as count FROM books")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as usize;

        let unique_words = sqlx::query("SELECT COUNT(DISTINCT word) as count FROM word_index")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as usize;

        Ok((total_books, unique_words))
    }

    async fn test_connection(&self) -> Result<(), StorageError> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}
