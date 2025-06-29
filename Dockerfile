# Stage 1: Base image with cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
	lld clang \
	protobuf-compiler \
	libprotobuf-dev \
	&& cargo install sccache \
	&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*
WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder

# Set sccache
ENV RUSTC_WRAPPER=sccache

COPY --from=planner /app/recipe.json recipe.json

# Build and cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the full project for final build
COPY . .

# Build the application
RUN cargo build --release --package articles-service --package users-service --package axum-otel-demo

# Copy all binaries to a location that can be accessed by the runtime stage
RUN cp /app/target/release/articles-service /app/articles-service && \
    cp /app/target/release/users-service /app/users-service && \
    cp /app/target/release/axum-otel-demo /app/axum-otel-demo

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates \
	openssl \
	libssl3 \
	&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*

# Copy all built binaries from the builder stage
COPY --from=builder /app/articles-service /usr/local/bin/articles-service
COPY --from=builder /app/users-service /usr/local/bin/users-service
COPY --from=builder /app/axum-otel-demo /usr/local/bin/axum-otel-demo

# Expose the application port
EXPOSE 8080