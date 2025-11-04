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

COPY --from=builder /app/target/release/indexing-service /usr/local/bin/indexing-service

EXPOSE 7002

CMD ["indexing-service"]
