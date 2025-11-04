FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY benches ./benches

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/search-service /usr/local/bin/search-service

EXPOSE 7003

CMD ["search-service"]