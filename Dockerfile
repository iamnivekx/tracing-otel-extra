# Stage 1: Base image with cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0 AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
	lld clang \
	protobuf-compiler \
	libprotobuf-dev \
	&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*
WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
COPY . .

# Build argument for package names (comma-separated for parsing)
ARG PACKAGE_NAMES

# Debug: show what packages we're trying to prepare
RUN echo "Preparing packages: ${PACKAGE_NAMES}"

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build profile, release by default
ENV RUSTC_WRAPPER=sccache

# Build argument for package names
ARG PACKAGE_NAMES

# Build and cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the full project for final build
COPY . .

# Build each package separately
RUN for package in $(echo ${PACKAGE_NAMES} | tr ',' ' '); do \
    echo "Building package: $package"; \
    cargo build --release --package "$package"; \
    done
