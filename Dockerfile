# Multi-stage Dockerfile for Loa'a with optimized caching
# Uses cargo-chef to cache dependencies separately from application code

# 1. Prepare recipe for dependency caching
FROM rustlang/rust:nightly-bookworm-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

# 2. Compute dependency recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

# 3. Build dependencies (this layer is cached unless dependencies change)
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    perl \
    make \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Install WASM target for Leptos frontend compilation
RUN rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli matching the version in Cargo.lock (0.2.105)
RUN cargo install wasm-bindgen-cli --version 0.2.105

# Install cargo-leptos for building the web application
RUN cargo install cargo-leptos

# Copy dependency recipe and build dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# 4. Build application (dependencies are already cached)
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the web application (includes embedded MCP server capability)
WORKDIR /app/crates/web
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo leptos build --release && \
    cp /app/target/release/loaa-web /tmp/loaa-web && \
    cp -r /app/target/site /tmp/site

# Build the standalone MCP server
WORKDIR /app/crates/mcp
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/loaa-mcp /tmp/loaa-mcp

# 5. Runtime stage (minimal final image)
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
COPY --from=builder /tmp/loaa-web ./loaa-web
COPY --from=builder /tmp/loaa-mcp ./loaa-mcp
COPY --from=builder /tmp/site ./site

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
