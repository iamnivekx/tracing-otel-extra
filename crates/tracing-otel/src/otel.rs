use anyhow::{Context, Result};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};

// Get resource with service name and attributes
pub(crate) fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(attributes.to_vec())
        .build()
}

/// Construct TracerProvider for OpenTelemetryLayer
pub(crate) fn init_tracer_provider(
    resource: &Resource,
    sample_ratio: f64,
) -> Result<SdkTracerProvider> {
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
pub(crate) fn init_meter_provider(
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
