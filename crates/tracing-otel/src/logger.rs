use crate::{
    guard::ProviderGuard,
    layer::{init_env_filter, init_format_layer, LogFormat},
};
use anyhow::{Context, Result};
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use tracing::Level;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "env")]
use crate::layer::{parse_attributes, parse_log_format};
#[cfg(feature = "env")]
use clap::Parser;

/// Configuration for the OpenTelemetry tracing and logging system.
///
/// This struct provides a builder-style API for configuring various aspects of
/// the tracing system, including:
/// - Service name and attributes
/// - Log format and level
/// - Sampling ratio
/// - Metrics collection settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "env", derive(Parser))]
#[cfg_attr(feature = "env", command(author, version, about, long_about = None))]
pub struct Logger {
    #[cfg_attr(
        feature = "env",
        arg(long, env = "SERVICE_NAME", default_value = env!("CARGO_CRATE_NAME"))
    )]
    pub service_name: String,
    #[cfg_attr(
        feature = "env",
        arg(long, env = "LOG_FORMAT", value_parser = parse_log_format, default_value = "compact")
    )]
    pub format: LogFormat,
    #[cfg_attr(feature = "env", arg(long, env = "LOG_ANSI", default_value = "true"))]
    pub ansi: bool,
    #[cfg_attr(feature = "env", arg(long, env = "LOG_LEVEL", default_value = "info"))]
    pub level: Level,
    #[cfg_attr(
        feature = "env",
        arg(long, env = "SAMPLE_RATIO", default_value = "1.0")
    )]
    pub sample_ratio: f64,
    #[cfg_attr(
        feature = "env",
        arg(long, env = "METRICS_INTERVAL_SECS", default_value = "30")
    )]
    pub metrics_interval_secs: u64,
    #[cfg_attr(
        feature = "env",
        arg(long, env = "ENABLE_STDOUT_METRICS", default_value = "true")
    )]
    pub enable_stdout_metrics: bool,
    #[cfg_attr(
        feature = "env",
        arg(long, env = "ATTRIBUTES", value_parser = parse_attributes, default_missing_value="")
    )]
    pub attributes: Vec<KeyValue>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            service_name: env!("CARGO_CRATE_NAME").to_string(),
            format: LogFormat::default(),
            ansi: true,
            level: Level::INFO,
            sample_ratio: 1.0,
            metrics_interval_secs: 30,
            enable_stdout_metrics: true,
            attributes: vec![],
        }
    }
}

impl Logger {
    /// Create a new configuration with the given service name
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    #[cfg(feature = "env")]
    pub fn from_env() -> Self {
        Self::parse()
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

    /// Set whether to enable stdout metrics output
    pub fn with_enable_stdout_metrics(mut self, enable: bool) -> Self {
        self.enable_stdout_metrics = enable;
        self
    }

    /// Add custom attributes to the resource
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Initialize tracing with this configuration
    pub fn init(self) -> Result<ProviderGuard> {
        init_tracing(self)
    }
}

// Get resource with service name and attributes
pub(crate) fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .with_attributes(attributes.to_vec())
        .build()
}

// Construct TracerProvider for OpenTelemetryLayer
pub(crate) fn init_tracer_provider(
    resource: &Resource,
    sample_ratio: f64,
) -> Result<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .context("Failed to build OTLP exporter")?;

    Ok(SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sample_ratio,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build())
}

// Construct MeterProvider for MetricsLayer
pub(crate) fn init_meter_provider(
    resource: &Resource,
    metrics_interval_secs: u64,
    enable_stdout_metrics: bool,
) -> Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .context("Failed to build OTLP exporter")?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(metrics_interval_secs))
        .build();

    let mut meter_builder = MeterProviderBuilder::default()
        .with_resource(resource.clone())
        .with_reader(reader);

    if enable_stdout_metrics {
        let stdout_reader =
            PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();
        meter_builder = meter_builder.with_reader(stdout_reader);
    }

    let meter_provider = meter_builder.build();
    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

/// Initialize tracing and OpenTelemetry with the given configuration
pub fn init_tracing(cfg: Logger) -> Result<ProviderGuard> {
    // Build resource with service name and additional attributes
    let resource = get_resource(&cfg.service_name, &cfg.attributes);
    let tracer_provider = init_tracer_provider(&resource, cfg.sample_ratio)?;
    let meter_provider = init_meter_provider(
        &resource,
        cfg.metrics_interval_secs,
        cfg.enable_stdout_metrics,
    )?;

    // Set up format layer
    let env_filter = init_env_filter(&cfg.level);
    let format_layer = init_format_layer(std::io::stdout, cfg.format, cfg.ansi);

    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(cfg.service_name.clone());
    let metrics_layer = MetricsLayer::new(meter_provider.clone());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(format_layer)
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
