use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct IngestResponse {
    book_id: u32,
    status: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    book_id: u32,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexResponse {
    book_id: u32,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    count: usize,
    books: Vec<u32>,
}

const INGESTION_SERVICE_URL: &str = "http://0.0.0.0:7001";
const INDEXING_SERVICE_URL: &str = "http://0.0.0.0:7002";
const SEARCH_SERVICE_URL: &str = "http://0.0.0.0:7003";

struct ControlModule {
    client: Client,
}

impl ControlModule {
    fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn wait_for_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Waiting for services to be ready...");

        let services = [
            ("Ingestion", format!("{}/status", INGESTION_SERVICE_URL)),
            ("Indexing", format!("{}/status", INDEXING_SERVICE_URL)),
            ("Search", format!("{}/status", SEARCH_SERVICE_URL)),
        ];

        for (name, url) in &services {
            loop {
                match self.client.get(url).send().await {
                    Ok(response) if response.status().is_success() => {
                        info!("{} service is ready", name);
                        break;
                    }
                    Ok(response) => {
                        warn!(
                            "{} service responded with status: {}",
                            name,
                            response.status()
                        );
                    }
                    Err(e) => {
                        warn!("{} service not ready: {}", name, e);
                    }
                }
                sleep(Duration::from_secs(2)).await;
            }
        }

        info!("All services are ready!");
        Ok(())
    }

    async fn ingest_book(
        &self,
        book_id: u32,
    ) -> Result<IngestResponse, Box<dyn std::error::Error>> {
        info!("Ingesting book {}", book_id);

        let url = format!("{}/ingest/{}", INGESTION_SERVICE_URL, book_id);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            let ingest_response: IngestResponse = response.json().await?;
            info!(
                "Successfully ingested book {}: {}",
                book_id, ingest_response.status
            );
            Ok(ingest_response)
        } else {
            let error_msg = format!("Failed to ingest book {}: {}", book_id, response.status());
            error!("{}", error_msg);
            Err(error_msg.into())
        }
    }

    async fn check_ingestion_status(
        &self,
        book_id: u32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let url = format!("{}/ingest/status/{}", INGESTION_SERVICE_URL, book_id);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let status_response: StatusResponse = response.json().await?;
            Ok(status_response.status == "available")
        } else {
            Ok(false)
        }
    }

    async fn index_book(&self, book_id: u32) -> Result<IndexResponse, Box<dyn std::error::Error>> {
        info!("Indexing book {}", book_id);

        let url = format!("{}/index/update/{}", INDEXING_SERVICE_URL, book_id);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            let index_response: IndexResponse = response.json().await?;
            info!(
                "Successfully indexed book {}: {}",
                book_id, index_response.status
            );
            Ok(index_response)
        } else {
            let error_msg = format!("Failed to index book {}: {}", book_id, response.status());
            error!("{}", error_msg);
            Err(error_msg.into())
        }
    }

    async fn get_available_books(&self) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let url = format!("{}/ingest/list", INGESTION_SERVICE_URL);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let list_response: ListResponse = response.json().await?;
            Ok(list_response.books)
        } else {
            Ok(Vec::new())
        }
    }

    async fn process_book(&self, book_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting processing pipeline for book {}", book_id);

        info!("📥 Step 1: Ingesting book {}", book_id);
        let ingest_response = self.ingest_book(book_id).await?;

        info!("⏳ Step 2: Waiting for ingestion confirmation...");
        sleep(Duration::from_millis(500)).await;

        info!("✅ Step 3: Verifying ingestion status...");
        if !self.check_ingestion_status(book_id).await? {
            return Err(format!(
                "Book {} ingestion verification failed - status not 'available'",
                book_id
            )
            .into());
        }
        info!(
            "✅ Book {} successfully ingested at: {}",
            book_id, ingest_response.path
        );

        info!("📊 Step 4: Indexing book {}", book_id);
        let index_response = self.index_book(book_id).await?;

        info!("✅ Step 5: Verifying indexing completion...");
        if index_response.status != "indexed" {
            return Err(format!(
                "Book {} indexing verification failed - status: {}",
                book_id, index_response.status
            )
            .into());
        }

        info!(
            "🎉 Successfully completed processing pipeline for book {}",
            book_id
        );
        Ok(())
    }

    async fn run_pipeline(&self, book_ids: Vec<u32>) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting pipeline for {} books", book_ids.len());

        for book_id in book_ids {
            match self.process_book(book_id).await {
                Ok(()) => info!("✓ Book {} processed successfully", book_id),
                Err(e) => error!("✗ Failed to process book {}: {}", book_id, e),
            }

            sleep(Duration::from_millis(100)).await;
        }

        info!("Pipeline execution complete");
        Ok(())
    }

    async fn continuous_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting continuous monitoring mode...");

        loop {
            sleep(Duration::from_secs(30)).await;

            match self.get_available_books().await {
                Ok(books) => {
                    if !books.is_empty() {
                        info!(
                            "Found {} books, checking if indexing is needed",
                            books.len()
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to get book list: {}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("control_module=info")
        .init();

    let control = ControlModule::new();

    // Wait for all services to be ready
    control.wait_for_services().await?;

    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--continuous" {
        // Run in continuous monitoring mode
        control.continuous_mode().await?;
    } else if args.len() > 1 {
        // Process specific book IDs from command line
        let book_ids: Result<Vec<u32>, _> = args[1..].iter().map(|s| s.parse()).collect();
        match book_ids {
            Ok(ids) => {
                control.run_pipeline(ids).await?;
            }
            Err(e) => {
                error!("Invalid book IDs provided: {}", e);
                info!("Usage: control-module [book_id1] [book_id2] ... or --continuous");
                std::process::exit(1);
            }
        }
    } else {
        // Default: process a few sample books
        let default_books = vec![1342, 84, 11, 74, 1080];
        info!(
            "No book IDs specified, processing default books: {:?}",
            default_books
        );
        control.run_pipeline(default_books).await?;
    }

    Ok(())
}
