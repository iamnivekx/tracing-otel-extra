#![deny(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![doc(html_root_url = "https://docs.rs/axum-otel/0.1.0")] // TODO: Update version
// Removed: #![doc = include_str!("../README.md")]

//! # axum-otel: OpenTelemetry Tracing for Axum Web Framework
//!
//! This crate provides a middleware for Axum web framework that automatically instruments HTTP requests
//! and responses, and adds OpenTelemetry tracing to the request and response spans.
//!
mod otel;
mod otel_span;
mod request_id;

// Exports for the tower-http::trace::TraceLayer based middleware
pub use otel_span::AxumOtelOnFailure;
pub use otel_span::AxumOtelOnResponse;
pub use otel_span::AxumOtelSpanCreator;
pub use tracing::Level;
