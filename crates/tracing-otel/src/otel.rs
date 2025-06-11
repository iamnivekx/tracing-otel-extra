use anyhow::{Context, Result};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use tracing::Level;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

// Get resource with service name and attributes
pub fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(attributes.to_vec())
        .build()
}

/// Construct TracerProvider for OpenTelemetryLayer
pub fn init_tracer_provider(resource: &Resource, sample_ratio: f64) -> Result<SdkTracerProvider> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .context("Failed to build OTLP exporter")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sample_ratio,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}

/// Construct MeterProvider for MetricsLayer
pub fn init_meter_provider(
    resource: &Resource,
    metrics_interval_secs: u64,
) -> Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .context("Failed to build OTLP exporter")?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(metrics_interval_secs))
        .build();

    let meter_builder = MeterProviderBuilder::default()
        .with_resource(resource.clone())
        .with_reader(reader);

    let meter_provider = meter_builder.build();
    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

// Initialize env filter from level
pub fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}

/// Initialize tracing and OpenTelemetry with the given configuration
pub fn setup_tracing<S>(
    service_name: &str,
    attributes: &[KeyValue],
    sample_ratio: f64,
    metrics_interval_secs: u64,
    level: Level,
    fmt_layer: S,
) -> Result<(SdkTracerProvider, SdkMeterProvider)>
where
    S: tracing_subscriber::Layer<Registry> + Send + Sync + 'static,
{
    // Build resource with service name and additional attributes
    let resource = get_resource(service_name, attributes);
    let tracer_provider = init_tracer_provider(&resource, sample_ratio)?;
    let meter_provider = init_meter_provider(&resource, metrics_interval_secs)?;

    // Set up env filter
    let env_filter = init_env_filter(&level);

    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(service_name.to_string());
    let metrics_layer = MetricsLayer::new(meter_provider.clone());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(metrics_layer)
        .with(otel_layer)
        .with(env_filter)
        .init();

    Ok((tracer_provider, meter_provider))
}
