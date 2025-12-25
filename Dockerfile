# Build Stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy actual source code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx .sqlx
ENV SQLX_OFFLINE=true

# Touch main.rs to force rebuild
RUN touch src/main.rs
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary and configuration
COPY --from=builder /usr/src/app/target/release/agentkey_backend .
# Copy migrations if needed for runtime migration (optional, typically handled by separate migration container or app startup)
# COPY --from=builder /usr/src/app/migrations ./migrations

# Expose port
EXPOSE 8080

# Environment variables
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

CMD ["./agentkey_backend"]
