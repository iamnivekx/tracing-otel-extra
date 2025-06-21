//! OpenTelemetry logging configuration and initialization.
//!
//! This module provides a flexible and configurable logging system that integrates
//! OpenTelemetry tracing and metrics. It offers both programmatic configuration
//! through a builder pattern and environment variable-based configuration.
//!
//! # Features
//!
//! - Builder-style configuration API
//! - Environment variable support (with "env" feature)
//! - Multiple log formats (compact, pretty, json)
//! - Configurable sampling and metrics collection
//! - Custom resource attributes
//!
//! # Quick Start
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//! use tracing::Level;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Basic initialization
//!     let guard = Logger::new("my-service").init()?;
//!
//!     // Your application code here...
//!
//!     // The guard will automatically clean up when dropped
//!     Ok(())
//! }
//! ```
//!
//! # Advanced Configuration
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//! use tracing::Level;
//! use opentelemetry::KeyValue;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let guard = Logger::new("my-service")
//!         .with_level(Level::DEBUG)
//!         .with_sample_ratio(0.5)
//!         .with_metrics_interval_secs(60)
//!         .with_attributes(vec![
//!             KeyValue::new("environment", "production"),
//!             KeyValue::new("version", "1.0.0"),
//!         ])
//!         .init()?;
//!
//!     // Your application code here...
//!
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables
//!
//! When using the "env" feature, you can configure the logger through environment variables:
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     #[cfg(feature = "env")]
//!     {
//!         // Using default prefix "LOG_"
//!         // let guard = Logger::from_env(None)?.init()?;
//!         // Or with custom prefix
//!         let guard = Logger::from_env(Some("MY_APP_"))?.init()?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Available Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `LOG_SERVICE_NAME` | Service name | Crate name |
//! | `LOG_FORMAT` | Log format (`compact`, `pretty`, `json`) | `compact` |
//! | `LOG_SPAN_EVENTS` | Span events (`FMT::NEW`, `FMT::ENTER`, `FMT::EXIT`, `FMT::CLOSE`, `FMT::NONE`, `FMT::ACTIVE`, `FMT::FULL`) | `FMT::NEW | FMT::CLOSE` |
//! | `LOG_ANSI` | Enable ANSI colors | `true` |
//! | `LOG_LEVEL` | Log level | `info` |
//! | `LOG_SAMPLE_RATIO` | Sampling ratio (0.0-1.0) | `1.0` |
//! | `LOG_METRICS_INTERVAL_SECS` | Metrics collection interval | `30` |
//! | `LOG_ATTRIBUTES` | Additional attributes (`key=value,key2=value2`) | - |
//!
//! # Examples
//!
//! ## Basic Configuration
//! ```bash
//! LOG_SERVICE_NAME=my-service
//! LOG_LEVEL=debug
//! ```
//!
//! ## Advanced Configuration
//! ```bash
//! LOG_FORMAT=json
//! LOG_ANSI=false
//! LOG_SAMPLE_RATIO=0.5
//! LOG_METRICS_INTERVAL_SECS=60
//! LOG_ATTRIBUTES=environment=prod,region=us-west
//! ```
use super::{
    layer::{
        deserialize_attributes, deserialize_level, deserialize_log_format, init_format_layer,
        LogFormat,
    },
    subscriber::setup_tracing,
};
use crate::otel::OtelGuard;
use anyhow::{Context, Result};
use opentelemetry::KeyValue;
use serde::Deserialize;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

/// Configuration for the OpenTelemetry tracing and logging system.
///
/// This struct provides a builder-style API for configuring various aspects of
/// the tracing system. It supports both programmatic configuration and
/// environment variable-based configuration (with the "env" feature).
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::{Logger, FmtSpan};
/// use tracing::Level;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create with default settings
///     // let guard = Logger::new("my-service").init()?;
///
///     // Create with custom settings
///     let guard = Logger::new("my-service")
///         .with_level(Level::DEBUG)
///         .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
///         .with_sample_ratio(0.5)
///         .init()?;
///
///     // Your application code here...
///
///     Ok(())
/// }
/// ```
///
/// # Environment Variables
///
/// When using the "env" feature, you can configure the logger through environment variables.
/// See the module-level documentation for a complete list of available variables.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Logger {
    /// The name of the service being traced.
    /// Defaults to the crate name if not specified.
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// The format to use for log output.
    /// Supported formats: compact, pretty, json.
    #[serde(
        deserialize_with = "deserialize_log_format",
        default = "LogFormat::default"
    )]
    pub format: LogFormat,

    /// The span events to include in the output.
    #[serde(
        default = "default_span_events",
        deserialize_with = "deserialize_span_events"
    )]
    pub span_events: FmtSpan,

    /// Whether to use ANSI colors in the output.
    /// Defaults to true.
    #[serde(default)]
    pub ansi: bool,

    /// The minimum log level to record.
    /// Defaults to INFO.
    #[serde(deserialize_with = "deserialize_level", default = "default_level")]
    pub level: Level,

    /// The ratio of traces to sample (0.0 to 1.0).
    /// Defaults to 1.0 (sample all traces).
    #[serde(default = "default_sample_ratio")]
    pub sample_ratio: f64,

    /// The interval in seconds between metrics collection.
    /// Defaults to 30 seconds.
    #[serde(default = "default_metrics_interval_secs")]
    pub metrics_interval_secs: u64,

    /// Additional attributes to add to the resource.
    /// These will be included in all traces and metrics.
    #[serde(default, deserialize_with = "deserialize_attributes")]
    pub attributes: Vec<KeyValue>,
}

fn deserialize_span_events<'de, D>(deserializer: D) -> Result<FmtSpan, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim();

    if s.is_empty() {
        return Ok(FmtSpan::NONE);
    }

    let mut result = FmtSpan::NONE;

    for part in s.split('|').map(|p| p.trim()) {
        let span = match part {
            "FMT::NEW" | "FmtSpan::NEW" => FmtSpan::NEW,
            "FMT::ENTER" | "FmtSpan::ENTER" => FmtSpan::ENTER,
            "FMT::EXIT" | "FmtSpan::EXIT" => FmtSpan::EXIT,
            "FMT::CLOSE" | "FmtSpan::CLOSE" => FmtSpan::CLOSE,
            "FMT::ACTIVE" | "FmtSpan::ACTIVE" => FmtSpan::ACTIVE,
            "FMT::FULL" | "FmtSpan::FULL" => return Ok(FmtSpan::FULL),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid span events: '{}'. Valid options: FMT::NEW, FMT::ENTER, FMT::EXIT, FMT::CLOSE, FMT::NONE, FMT::ACTIVE, FMT::FULL",
                    part
                )));
            }
        };
        result |= span;
    }

    Ok(result)
}

#[inline]
fn default_service_name() -> String {
    env!("CARGO_CRATE_NAME").to_string()
}

#[inline]
fn default_level() -> Level {
    Level::INFO
}

#[inline]
fn default_sample_ratio() -> f64 {
    1.0
}

#[inline]
fn default_metrics_interval_secs() -> u64 {
    30
}

#[inline]
fn default_span_events() -> FmtSpan {
    FmtSpan::NEW | FmtSpan::CLOSE
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            format: LogFormat::default(),
            span_events: default_span_events(),
            ansi: true,
            level: default_level(),
            sample_ratio: default_sample_ratio(),
            metrics_interval_secs: 30,
            attributes: vec![],
        }
    }
}

impl Logger {
    /// Create a new configuration with the given service name.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service being traced
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set the service name.
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the log format (compact, pretty, or json).
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the span events to include in the output.
    pub fn with_span_events(mut self, span_events: FmtSpan) -> Self {
        self.span_events = span_events;
        self
    }

    /// Set whether to use ANSI colors in the output.
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }

    /// Set the minimum log level to record.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the ratio of traces to sample (0.0 to 1.0).
    pub fn with_sample_ratio(mut self, ratio: f64) -> Self {
        self.sample_ratio = ratio;
        self
    }

    /// Set the interval in seconds between metrics collection.
    pub fn with_metrics_interval_secs(mut self, secs: u64) -> Self {
        self.metrics_interval_secs = secs;
        self
    }

    /// Add custom attributes to the resource.
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Initialize tracing with this configuration.
    ///
    /// This method will:
    /// 1. Set up the global tracing subscriber
    /// 2. Configure the OpenTelemetry tracer and meter providers
    /// 3. Return a guard that ensures proper cleanup
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `ProviderGuard` that will automatically
    /// clean up the tracing providers when dropped.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // Create with default settings
    ///     let guard = Logger::new("my-service").init()?;
    ///     
    ///     // Use tracing...
    ///     tracing::info!("Hello, world!");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Advanced configuration:
    /// ```rust
    /// use tracing_otel_extra::Logger;
    /// use tracing::Level;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // Create with custom settings
    ///     let guard = Logger::new("my-service")
    ///         .with_level(Level::DEBUG)
    ///         .with_sample_ratio(0.5)
    ///         .with_metrics_interval_secs(30)
    ///         .init()?;
    ///     
    ///     // Use tracing with custom configuration
    ///     tracing::debug!("Debug message");
    ///     tracing::info!("Info message");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to initialize the tracing subscriber
    /// - Failed to set up OpenTelemetry providers
    /// - Failed to configure the environment filter
    pub fn init(self) -> Result<OtelGuard> {
        init_tracing_from_logger(self)
    }

    /// Initialize the logger from environment variables.
    ///
    /// This method requires the "env" feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Optional prefix for environment variables. If None, "LOG_" is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     #[cfg(feature = "env")]
    ///     {
    ///         let guard = Logger::from_env(None)?.init()?;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "env")]
    pub fn from_env(prefix: Option<&str>) -> Result<Self> {
        init_logger_from_env(prefix)
    }
}

// Initialize tracing from logger
pub fn init_tracing_from_logger(logger: Logger) -> Result<OtelGuard> {
    let guard = setup_tracing(
        &logger.service_name,
        &logger.attributes,
        logger.sample_ratio,
        logger.metrics_interval_secs,
        logger.level,
        init_format_layer(logger.format, logger.ansi, logger.span_events),
    )
    .context("Failed to initialize tracing")?;
    Ok(guard)
}

/// Convenience function to initialize tracing with default settings
pub fn init_logging(service_name: &str) -> Result<OtelGuard> {
    let logger = Logger::new(service_name);
    init_tracing_from_logger(logger)
}

#[cfg(feature = "env")]
pub fn init_logger_from_env(prefix: Option<&str>) -> Result<Logger> {
    let prefix = prefix.unwrap_or("LOG_");
    let logger = envy::prefixed(prefix)
        .from_env()
        .context("Failed to deserialize environment variables")?;
    Ok(logger)
}

#[cfg(feature = "env")]
pub fn init_logging_from_env(prefix: Option<&str>) -> Result<OtelGuard> {
    let logger = init_logger_from_env(prefix)?;
    init_tracing_from_logger(logger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;
    use serde_json;

    #[derive(Debug, serde::Deserialize)]
    struct TestFmtSpan {
        #[serde(deserialize_with = "deserialize_span_events")]
        span_events: FmtSpan,
    }

    #[test]
    fn test_logger_builder() {
        let logger = Logger::new("test-service")
            .with_level(Level::DEBUG)
            .with_sample_ratio(0.5)
            .with_attributes(vec![KeyValue::new("test", "value")]);

        assert_eq!(logger.service_name, "test-service");
        assert_eq!(logger.level, Level::DEBUG);
        assert_eq!(logger.sample_ratio, 0.5);
        assert_eq!(logger.attributes.len(), 1);
    }

    #[test]
    fn test_deserialize_span_events_fmt_format() {
        // Test single values
        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::NEW"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::CLOSE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::CLOSE);

        // Test combinations
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::NEW|FMT::CLOSE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW | FmtSpan::CLOSE);

        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::ENTER|FMT::EXIT"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::ENTER | FmtSpan::EXIT);

        // Test predefined combinations
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::ACTIVE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::ACTIVE);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::FULL"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::FULL);

        // Test with spaces
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": " FMT::NEW | FMT::CLOSE "}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW | FmtSpan::CLOSE);
    }

    #[test]
    fn test_deserialize_span_events_empty() {
        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": ""}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NONE);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "   "}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NONE);
    }

    #[test]
    fn test_deserialize_span_events_invalid() {
        let result: Result<TestFmtSpan, _> = serde_json::from_str(r#"{"span_events": "INVALID"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        println!("Error message: {}", error_msg);
        assert!(error_msg.contains("Invalid span events"));

        let result: Result<TestFmtSpan, _> =
            serde_json::from_str(r#"{"span_events": "FMT::NEW|INVALID"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        println!("Error message: {}", error_msg);
        assert!(error_msg.contains("Invalid span events"));
    }
}
