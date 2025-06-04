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
#![doc(html_root_url = "https://docs.rs/axum-otel/0.29.0")]
#![macro_use]
#![allow(unused_imports)]

//! # axum-otel: OpenTelemetry Tracing for Axum Web Framework
//!
//! This crate provides a middleware for Axum web framework that automatically instruments HTTP requests
//! and responses, and adds OpenTelemetry tracing to the request and response spans.
//!
//! ## Features
//!
//! - Automatic request and response tracing
//! - OpenTelemetry integration
//! - Request ID tracking
//! - Customizable span attributes
//! - Error tracking
//!
//! ## Usage
//!
//! ```rust
//! use axum::{
//!     routing::get,
//!     Router,
//! };
//! use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
//! use tower_http::trace::TraceLayer;
//!
//! async fn handler() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! // Build our application with a route
//! let app: Router<()> = Router::new()
//!     .route("/", get(handler))
//!     .layer(
//!         TraceLayer::new_for_http()
//!             .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
//!             .on_response(AxumOtelOnResponse::new().level(Level::INFO))
//!             .on_failure(AxumOtelOnFailure::new()),
//!     );
//! ```
//!
//! ## Components
//!
//! ### AxumOtelSpanCreator
//!
//! Creates spans for each request with relevant HTTP information. This is used with
//! `TraceLayer::make_span_with`.
//!
//! Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/635692d757f29dfa3041c02cd66c195be07bc8b3/tower-http/src/trace/mod.rs#L414).
//!
//! The span will include the following attributes:
//!
//! - `http.method`: The HTTP method
//! - `http.route`: The matched route
//! - `http.status_code`: The response status code
//! - `http.client_ip`: The client's IP address
//! - `http.user_agent`: The User-Agent header
//! - `http.host`: The Host header
//! - `request_id`: A unique request identifier, if set
//! - `trace_id`: The OpenTelemetry trace ID
//!
//! ### AxumOtelOnResponse
//!
//! Records response status and latency. This is used with `TraceLayer::on_response`.
//!
//! Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/635692d757f29dfa3041c02cd66c195be07bc8b3/tower-http/src/trace/mod.rs#L414).
//!
//! The following attributes are added to the span:
//!
//! - `http.status_code`: The response status code
//! - `otel.status_code`: The OpenTelemetry status code (OK for successful responses)
//!
//! ### AxumOtelOnFailure
//!
//! Handles error cases and updates span status. This is used with `TraceLayer::on_failure`.
//!
//! Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/635692d757f29dfa3041c02cd66c195be07bc8b3/tower-http/src/trace/mod.rs#L414).
//!
//! When a server error occurs, the span's `otel.status_code` is set to "ERROR".
//!
//! ## Examples
//!
//! See the [examples](./examples) directory for complete examples.

mod event_macro;
mod make_span;
mod on_failure;
mod on_response;
mod otel;
mod request_id;

// crate private exports
pub(crate) use otel::set_otel_parent;
pub(crate) use request_id::get_request_id;

// Exports for the tower-http::trace::TraceLayer based middleware
pub use make_span::AxumOtelSpanCreator;
pub use on_failure::AxumOtelOnFailure;
pub use on_response::AxumOtelOnResponse;

// Re-export the Level enum from tracing crate
pub use tracing::Level;
