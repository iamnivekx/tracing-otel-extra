use crate::{
    guard::ProviderGuard,
    layer::{init_env_filter, init_format_layer, LogFormat},
};
use anyhow::Result;
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    resource::ResourceDetector,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use tracing::Level;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Logger configuration
#[derive(Debug, Clone)]
pub struct Logger {
    pub service_name: String,
    pub format: LogFormat,
    pub ansi: bool,
    pub level: Level,
    pub sample_ratio: f64,
    pub metrics_interval_secs: u64,
    pub enable_stdout_metrics: bool,
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
        self.sample_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set the metrics collection interval in seconds
    pub fn with_metrics_interval(mut self, secs: u64) -> Self {
        self.metrics_interval_secs = secs;
        self
    }

    /// Set whether to enable stdout metrics output
    pub fn with_stdout_metrics(mut self, enable: bool) -> Self {
        self.enable_stdout_metrics = enable;
        self
    }

    /// Initialize tracing with this configuration
    pub fn init(self) -> Result<ProviderGuard> {
        init_tracing(self)
    }

    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }
}

// Get resource with detectors and service name
pub(crate) fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    let detectors: Vec<Box<dyn ResourceDetector>> = vec![
        Box::new(OsResourceDetector),
        Box::new(ProcessResourceDetector),
    ];

    Resource::builder()
        .with_detectors(&detectors)
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .with_attributes(attributes.to_vec())
        .build()
}

// Construct TracerProvider for OpenTelemetryLayer
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#otlp-span-exporter
// OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
// OTEL_EXPORTER_OTLP_PROTOCOL=grpc
pub(crate) fn init_tracer_provider(resource: &Resource, sample_ratio: f64) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to build OTLP exporter");

    SdkTracerProvider::builder()
        // Customize sampling strategy
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sample_ratio,
        ))))
        // If export trace to AWS X-Ray, you can use XrayIdGenerator
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build()
}

// Construct MeterProvider for MetricsLayer
pub(crate) fn init_meter_provider(
    resource: &Resource,
    metrics_interval_secs: u64,
    enable_stdout_metrics: bool,
) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .expect("Failed to build OTLP exporter");

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(metrics_interval_secs))
        .build();

    let mut meter_builder = MeterProviderBuilder::default()
        .with_resource(resource.clone())
        .with_reader(reader);

    // For debugging in development
    if enable_stdout_metrics {
        let stdout_reader =
            PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();
        meter_builder = meter_builder.with_reader(stdout_reader);
    }

    let meter_provider = meter_builder.build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}

/// Initialize tracing and OpenTelemetry with the given configuration
pub fn init_tracing(cfg: Logger) -> Result<ProviderGuard> {
    // Build resource with service name and additional attributes
    let resource = get_resource(&cfg.service_name, &cfg.attributes);
    let tracer_provider = init_tracer_provider(&resource, cfg.sample_ratio);
    let meter_provider = init_meter_provider(
        &resource,
        cfg.metrics_interval_secs,
        cfg.enable_stdout_metrics,
    );

    // Set up format layer
    let env_filter = init_env_filter(&cfg.level);
    let format_layer = init_format_layer(std::io::stdout, cfg.format, cfg.ansi);

    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(cfg.service_name.clone());
    // Create a `tracing` layer to emit spans as structured logs to Metrics
    let metrics_layer = MetricsLayer::new(meter_provider.clone());
    // Set up telemetry layer with tracer
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

    let guard = init_tracing(logger)?;
    // Note: The guard is dropped here, which means providers will be shut down immediately
    // This maintains backward compatibility but is not ideal
    Ok(guard)
}
