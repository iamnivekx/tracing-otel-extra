//! # Tracing Extra
//!
//! This crate provides common utilities for initializing tracing and OpenTelemetry
//! in axum applications.
//!
//! ## Examples
//!
//! Basic usage with configuration builder:
//! ```rust,no_run
//! use tracing_extra::{Logger, Format};
//! use opentelemetry::KeyValue;
//!
//! #[tokio::main]
//! async fn main() {
//!     let _guard = Logger::new("my-service")
//!         .with_format(Format::Json)
//!         .with_ansi(false)
//!         .with_sample_ratio(0.1)
//!         .with_attributes(vec![
//!             KeyValue::new("environment", "production"),
//!             KeyValue::new("version", "1.0.0"),
//!         ])
//!         .init()
//!         .expect("Failed to initialize tracing");
//!     
//!     // Your application code here
//!     
//!     // Cleanup is handled automatically when the guard is dropped
//! }
//! ```
//!
//! Legacy usage (for backward compatibility):
//! ```rust,no_run
//! use tracing_extra::init_logging;
//!
//! #[tokio::main]
//! async fn main() {
//!     let _guard = init_logging("my-service").expect("Failed to initialize tracing");
//!     
//!     // Your application code here
//! }
//! ```
pub mod guard;
pub mod layer;
pub mod logger;

// Re-export the main types for convenience
pub use guard::ProviderGuard;

pub use layer::LogFormat;
pub use logger::{init_logging, Logger};
