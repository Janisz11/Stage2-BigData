use crate::utils::file::{create_datalake_path, header_body_split};
use std::fs;
use tracing::info;

pub async fn download_book(
    book_id: u32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://www.gutenberg.org/cache/epub/{}/pg{}.txt",
        book_id, book_id
    );
    let datalake_path = create_datalake_path(book_id);

    fs::create_dir_all(&datalake_path)?;

    let header_path = format!("{}/header_{}.txt", datalake_path, book_id);
    let body_path = format!("{}/body_{}.txt", datalake_path, book_id);

    if std::path::Path::new(&header_path).exists() && std::path::Path::new(&body_path).exists() {
        info!("Book {} already exists, skipping download", book_id);
        return Ok(datalake_path);
    }

    info!("Downloading book {} from {}", book_id, url);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download book {}: {}", book_id, response.status()).into());
    }

    let text = response.text().await?;
    let (header, body) = header_body_split(&text);

    fs::write(&header_path, header)?;
    fs::write(&body_path, body)?;

    info!(
        "Successfully downloaded book {} to {}",
        book_id, datalake_path
    );
    Ok(datalake_path)
}
