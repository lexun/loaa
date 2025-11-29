# Multi-stage Dockerfile for Loa'a
# Builds both web server and MCP server in a single image

FROM rustlang/rust:nightly-bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    perl \
    make \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-leptos for building the web application
RUN cargo install cargo-leptos

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the web application (includes embedded MCP server capability)
WORKDIR /app/crates/web
RUN cargo leptos build --release

# Build the standalone MCP server
WORKDIR /app/crates/mcp
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 loaa

# Set working directory
WORKDIR /app

# Copy compiled binaries from builder
COPY --from=builder /app/target/release/loaa-web ./loaa-web
COPY --from=builder /app/target/release/loaa-mcp ./loaa-mcp
COPY --from=builder /app/target/site ./site

# Copy static assets
COPY --from=builder /app/crates/web/style ./style

# Create data directory for embedded database
RUN mkdir -p /app/data && chown -R loaa:loaa /app

# Switch to app user
USER loaa

# Expose ports
EXPOSE 3000 3001

# Set default environment variables
ENV LOAA_DB_MODE=embedded
ENV LOAA_DB_PATH=/app/data/loaa.db
ENV LOAA_INCLUDE_MCP=true
ENV LOAA_MCP_PORT=3001
ENV RUST_LOG=info

# Default command: run web server in all-in-one mode
CMD ["./loaa-web"]
