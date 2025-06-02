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
//! let app = Router::new()
//!     .route("/", get(handler))
//!     .layer(
//!         TraceLayer::new_for_http()
//!             .make_span_with(AxumOtelSpanCreator)
//!             .on_response(AxumOtelOnResponse)
//!             .on_failure(AxumOtelOnFailure),
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
//! The following attributes are added to the span:
//!
//! - `http.status_code`: The response status code
//! - `otel.status_code`: The OpenTelemetry status code (OK for successful responses)
//!
//! ### AxumOtelOnFailure
//!
//! Handles error cases and updates span status. This is used with `TraceLayer::on_failure`.
//!
//! When a server error occurs, the span's `otel.status_code` is set to "ERROR".
//!
//! ## Examples
//!
//! See the [examples](./examples) directory for complete examples.

mod otel;
mod otel_span;
mod request_id;

// Exports for the tower-http::trace::TraceLayer based middleware
pub use otel_span::AxumOtelOnFailure;
pub use otel_span::AxumOtelOnResponse;
pub use otel_span::AxumOtelSpanCreator;
pub use tracing::Level;
