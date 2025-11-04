# Stage 2: Service-Oriented Architecture
## Search Engine Project - Big Data

This repository contains the Stage 2 implementation of the Search Engine Project, which transforms the monolithic Stage 1 data layer into a distributed service-oriented architecture using Rust microservices with Axum.

## Architecture Overview

The system consists of 3 independent microservices:

1. **Ingestion Service** (Port 7001) - Downloads books from Project Gutenberg
2. **Indexing Service** (Port 7002) - Processes books and builds search indexes
3. **Search Service** (Port 7003) - Provides REST API for querying books
4. **Control Module** - Orchestrates the pipeline workflow

## Services

### Ingestion Service (Port 7001)

**Endpoints:**
- `POST /ingest/{book_id}` - Download and store a book
- `GET /ingest/status/{book_id}` - Check if book is available
- `GET /ingest/list` - List all downloaded books
- `GET /status` - Health check

**Example:**
```bash
curl -X POST http://localhost:7001/ingest/1342
curl http://localhost:7001/ingest/status/1342
curl http://localhost:7001/ingest/list
```

### Indexing Service (Port 7002)

**Endpoints:**
- `POST /index/update/{book_id}` - Index a specific book
- `POST /index/rebuild` - Rebuild entire index
- `GET /index/status` - Get indexing statistics
- `GET /status` - Health check

**Example:**
```bash
curl -X POST http://localhost:7002/index/update/1342
curl -X POST http://localhost:7002/index/rebuild
curl http://localhost:7002/index/status
```

### Search Service (Port 7003)

**Endpoints:**
- `GET /search?q={term}` - Search for books
- `GET /search?q={term}&author={name}` - Search with author filter
- `GET /search?q={term}&language={code}` - Search with language filter
- `GET /search?q={term}&year={YYYY}` - Search with year filter
- `GET /status` - Health check

**Examples:**
```bash
curl "http://localhost:7003/search?q=adventure"
curl "http://localhost:7003/search?q=adventure&author=Jane%20Austen"
curl "http://localhost:7003/search?q=adventure&language=en"
curl "http://localhost:7003/search?q=adventure&year=1865"
curl "http://localhost:7003/search?q=adventure&author=Jules%20Verne&language=fr&year=1865"
```

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Rust 1.70+ (for local development)

### Running with Docker Compose

1. **Start the services:**
```bash
cd services
docker-compose up --build
```

2. **Run the control module to process default books:**
```bash
docker-compose --profile control up control-module
```

3. **Process specific books:**
```bash
docker-compose run --rm control-module control-module 1342 84 11
```

4. **Test the search API:**
```bash
curl "http://localhost:7003/search?q=pride"
```

### Running Locally

1. **Build all services:**
```bash
cd services
cargo build --release
```

2. **Start infrastructure:**
```bash
docker-compose up redis postgres
```

3. **Run services (in separate terminals):**
```bash
cd ingestion-service && cargo run
cd indexing-service && cargo run
cd search-service && cargo run
cd control-module && cargo run
```

## Development

### Project Structure
```
services/
├── ingestion-service/     # Downloads books to datalake
├── indexing-service/      # Processes and indexes books
├── search-service/        # Search API endpoints
├── control-module/        # Orchestration logic
└── docker-compose.yml     # Service configuration
```

### Running Tests
```bash
# Run all tests
cargo test

# Run specific service tests
cd ingestion-service && cargo test
cd indexing-service && cargo test
cd search-service && cargo test
```

### Running Benchmarks
```bash
# Run benchmarks for each service
cd ingestion-service && cargo bench
cd indexing-service && cargo bench
cd search-service && cargo bench
```

Benchmark results are saved to `target/criterion/` with HTML reports.

## API Examples

### Complete Workflow

1. **Ingest a book:**
```bash
curl -X POST http://localhost:7001/ingest/1342
# Response: {"book_id":1342,"status":"downloaded","path":"/app/datalake/20251104/42"}
```

2. **Index the book:**
```bash
curl -X POST http://localhost:7002/index/update/1342
# Response: {"book_id":1342,"index":"updated"}
```

3. **Search for the book:**
```bash
curl "http://localhost:7003/search?q=pride"
# Response: {"query":"pride","filters":{},"count":1,"results":[...]}
```

### Search Examples

```bash
# Basic search
curl "http://localhost:7003/search?q=adventure"

# Search by author
curl "http://localhost:7003/search?q=adventure&author=Jane%20Austen"

# Search by language
curl "http://localhost:7003/search?q=adventure&language=fr"

# Search by year
curl "http://localhost:7003/search?q=adventure&year=1865"

# Combined filters
curl "http://localhost:7003/search?q=adventure&author=Jules%20Verne&language=fr&year=1865"
```

## Data Flow

1. **Control Module** selects books to process
2. **Ingestion Service** downloads books from Project Gutenberg
3. Books are stored in `/app/datalake` with hierarchical structure
4. **Indexing Service** processes books and builds search indexes
5. **Search Service** queries the indexes for user searches

## Performance

The system includes Criterion benchmarks for:
- Text tokenization performance
- Metadata extraction speed
- Search query processing
- Index building operations

Run benchmarks with:
```bash
cargo bench --all
```

## Configuration

Services can be configured via environment variables:
- `PORT` - Service port (default: 7001, 7002, 7003)
- `RUST_LOG` - Logging level (default: info)

## Monitoring

Health check endpoints are available at `/status` for each service:
```bash
curl http://localhost:7001/status
curl http://localhost:7002/status
curl http://localhost:7003/status
```

## Stage 1 Integration

This Stage 2 implementation preserves the datalake structure from Stage 1:
- Hierarchical directory organization by date and book ID
- Header/body file separation
- Compatible with existing Python data layer

The system bridges the Python backend with Rust microservices to create a scalable, distributed architecture.