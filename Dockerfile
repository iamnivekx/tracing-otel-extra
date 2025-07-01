# Stage 1: Base image with cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef

WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build and cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the full project for final build
COPY . .

# Build the application
RUN cargo build --release --locked --package articles-service --package users-service --package axum-otel-demo

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime

# Install dependencies for OpenSSL
RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates \
	openssl \
	libssl3 \
	&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*

WORKDIR /app

# Copy all built binaries from the builder stage
COPY --from=builder /app/target/release/articles-service /usr/local/bin/articles-service
COPY --from=builder /app/target/release/users-service /usr/local/bin/users-service
COPY --from=builder /app/target/release/axum-otel-demo /usr/local/bin/axum-otel-demo

# Expose the application ports
EXPOSE 8080
EXPOSE 8081
EXPOSE 8082