use crate::{
    guard::ProviderGuard,
    layer::{
        deserialize_attributes, deserialize_level, deserialize_log_format, init_env_filter,
        init_format_layer, LogFormat,
    },
    otel::{get_resource, init_meter_provider, init_tracer_provider},
};
use anyhow::{Context, Result};
use opentelemetry::{trace::TracerProvider as _, KeyValue};
use tracing::Level;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for the OpenTelemetry tracing and logging system.
///
/// This struct provides a builder-style API for configuring various aspects of
/// the tracing system, including:
/// - Service name and attributes
/// - Log format and level
/// - Sampling ratio
/// - Metrics collection settings
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::Logger;
/// use tracing::Level;
///
/// // Create with default settings
/// let logger = Logger::new("my-service");
///
/// // Create with custom settings
/// let logger = Logger::new("my-service")
///     .with_level(Level::DEBUG)
///     .with_sample_ratio(0.5);
///
/// // Initialize from environment variables
/// #[cfg(feature = "env")]
/// {
///     // Using default prefix "LOG_"
///     let logger = Logger::from_env(None).unwrap();
///
///     // Using custom prefix "MY_APP_"
///     let logger = Logger::from_env(Some("MY_APP_")).unwrap();
/// }
/// ```
///
/// # Environment Variables
///
/// When using `from_env`, the following environment variables can be set:
///
/// - `LOG_SERVICE_NAME`: Service name (defaults to crate name)
/// - `LOG_FORMAT`: Log format, one of "compact", "pretty", or "json" (defaults to "compact")
/// - `LOG_ANSI`: Whether to use ANSI colors (defaults to true)
/// - `LOG_LEVEL`: Log level (defaults to "info")
/// - `LOG_SAMPLE_RATIO`: Sampling ratio from 0.0 to 1.0 (defaults to 1.0)
/// - `LOG_METRICS_INTERVAL_SECS`: Metrics collection interval in seconds (defaults to 30)
/// - `LOG_ATTRIBUTES`: Comma-separated list of key=value pairs for additional attributes
///
/// # Examples of Environment Variables
///
/// ```bash
/// # Basic configuration
/// LOG_SERVICE_NAME=my-service
/// LOG_LEVEL=debug
///
/// # Advanced configuration
/// LOG_FORMAT=json
/// LOG_ANSI=false
/// LOG_SAMPLE_RATIO=0.5
/// LOG_METRICS_INTERVAL_SECS=60
/// LOG_ATTRIBUTES=environment=prod,region=us-west
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Logger {
    /// The name of the service being traced
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// The format to use for log output
    #[serde(
        deserialize_with = "deserialize_log_format",
        default = "LogFormat::default"
    )]
    pub format: LogFormat,
    /// Whether to use ANSI colors in the output
    #[serde(default)]
    pub ansi: bool,
    /// The minimum log level to record
    #[serde(deserialize_with = "deserialize_level", default = "default_level")]
    pub level: Level,
    /// The ratio of traces to sample (0.0 to 1.0)
    #[serde(default = "default_sample_ratio")]
    pub sample_ratio: f64,
    /// The interval in seconds between metrics collection
    #[serde(default = "default_metrics_interval_secs")]
    pub metrics_interval_secs: u64,
    /// Additional attributes to add to the resource
    #[serde(default, deserialize_with = "deserialize_attributes")]
    pub attributes: Vec<KeyValue>,
}

fn default_service_name() -> String {
    env!("CARGO_CRATE_NAME").to_string()
}

fn default_level() -> Level {
    Level::INFO
}

fn default_sample_ratio() -> f64 {
    1.0
}

fn default_metrics_interval_secs() -> u64 {
    30
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            format: LogFormat::default(),
            ansi: true,
            level: default_level(),
            sample_ratio: default_sample_ratio(),
            metrics_interval_secs: 30,
            attributes: vec![],
        }
    }
}

impl Logger {
    /// Create a new configuration with the given service name
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service being traced
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// let logger = Logger::new("my-service");
    /// ```
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set the service name
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set whether to use ANSI colors
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }

    /// Set the log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the sampling ratio (0.0 to 1.0)
    pub fn with_sample_ratio(mut self, ratio: f64) -> Self {
        self.sample_ratio = ratio;
        self
    }

    /// Set the metrics collection interval in seconds
    pub fn with_metrics_interval_secs(mut self, secs: u64) -> Self {
        self.metrics_interval_secs = secs;
        self
    }

    /// Add custom attributes to the resource
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Initialize the logger from environment variables
    ///
    /// This method requires the "env" feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Optional prefix for environment variables. If None, "LOG_" is used.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the configured `Logger` or an error if the
    /// environment variables could not be parsed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_otel_extra::Logger;
    /// #[cfg(feature = "env")]
    /// {
    ///     // Using default prefix "LOG_"
    ///     let logger = Logger::from_env(None).unwrap();
    ///
    ///     // Using custom prefix "MY_APP_"
    ///     let logger = Logger::from_env(Some("MY_APP_")).unwrap();
    /// }
    /// ```
    #[cfg(feature = "env")]
    pub fn from_env(prefix: Option<&str>) -> Result<Self> {
        let prefix = prefix.unwrap_or("LOG_");
        let logger = envy::prefixed(prefix)
            .from_env()
            .context("Failed to deserialize environment configuration")?;
        Ok(logger)
    }

    /// Initialize tracing with this configuration
    pub fn init(self) -> Result<ProviderGuard> {
        init_tracing(self)
    }
}

/// Initialize tracing and OpenTelemetry with the given configuration
pub fn init_tracing(cfg: Logger) -> Result<ProviderGuard> {
    // Build resource with service name and additional attributes
    let resource = get_resource(&cfg.service_name, &cfg.attributes);
    let tracer_provider = init_tracer_provider(&resource, cfg.sample_ratio)?;
    let meter_provider = init_meter_provider(&resource, cfg.metrics_interval_secs)?;

    // Set up env filter
    let env_filter = init_env_filter(&cfg.level);
    // Set up format layer
    let fmt_layer = init_format_layer(cfg.format, cfg.ansi);
    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(cfg.service_name.clone());
    let metrics_layer = MetricsLayer::new(meter_provider.clone());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(metrics_layer)
        .with(otel_layer)
        .with(env_filter)
        .init();

    Ok(ProviderGuard::new(
        Some(tracer_provider),
        Some(meter_provider),
    ))
}

/// Convenience function to initialize tracing with default settings
pub fn init_logging(service_name: &str) -> Result<ProviderGuard> {
    let logger = Logger::new(service_name);
    init_tracing(logger)
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
pub fn init_logging_from_env(prefix: Option<&str>) -> Result<ProviderGuard> {
    let logger = init_logger_from_env(prefix)?;
    init_tracing(logger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;

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
}
