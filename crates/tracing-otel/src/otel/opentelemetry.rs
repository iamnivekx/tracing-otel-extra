//! OpenTelemetry integration for tracing.
//!
//! This module provides utilities for initializing and configuring OpenTelemetry
//! tracing and metrics in your application. It includes functions for:
//!
//! - Setting up tracer and meter providers
//! - Configuring resource attributes
//! - Initializing environment filters
//! - Setting up the complete tracing stack
//!
//! # Examples
//!
//! ```rust
//! use tracing_otel_extra::otel::{setup_tracing, get_resource};
//! use opentelemetry::KeyValue;
//! use tracing::Level;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let service_name = "my-service";
//!     let attributes = vec![
//!         KeyValue::new("environment", "production"),
//!         KeyValue::new("version", "1.0.0"),
//!     ];
//!
//!     let (tracer_provider, meter_provider) = setup_tracing(
//!         service_name,
//!         &attributes,
//!         1.0,  // sample ratio
//!         30,   // metrics interval in seconds
//!         Level::INFO,
//!         tracing_subscriber::fmt::layer(),
//!     )?;
//!
//!     // Your application code here...
//!
//!     // Cleanup when done
//!     tracer_provider.shutdown()?;
//!     meter_provider.shutdown()?;
//!     Ok(())
//! }
//! ```

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

/// Creates a resource with the given service name and attributes.
///
/// This function builds an OpenTelemetry resource that identifies your service
/// and includes any additional attributes you want to track.
///
/// # Arguments
///
/// * `service_name` - The name of your service
/// * `attributes` - Additional key-value pairs to include in the resource
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::otel::get_resource;
/// use opentelemetry::KeyValue;
///
/// let resource = get_resource(
///     "my-service",
///     &[KeyValue::new("environment", "production")],
/// );
/// ```
pub fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(attributes.to_vec())
        .build()
}

/// Initializes a tracer provider for OpenTelemetry tracing.
///
/// This function sets up a tracer provider with the following features:
/// - Parent-based sampling
/// - Random ID generation
/// - OTLP exporter
/// - Custom resource attributes
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use
/// * `sample_ratio` - The ratio of traces to sample (0.0 to 1.0)
///
/// # Returns
///
/// Returns a `Result` containing the configured `SdkTracerProvider` or an error
/// if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::otel::{get_resource, init_tracer_provider};
/// use opentelemetry::KeyValue;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let resource = get_resource("my-service", &[]);
///     let tracer_provider = init_tracer_provider(&resource, 1.0)?;
///     Ok(())
/// }
/// ```
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

/// Initializes a meter provider for OpenTelemetry metrics.
///
/// This function sets up a meter provider with the following features:
/// - Periodic metric collection
/// - OTLP exporter
/// - Custom resource attributes
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use
/// * `metrics_interval_secs` - The interval in seconds between metric collections
///
/// # Returns
///
/// Returns a `Result` containing the configured `SdkMeterProvider` or an error
/// if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::otel::{get_resource, init_meter_provider};
/// use opentelemetry::KeyValue;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let resource = get_resource("my-service", &[]);
///     let meter_provider = init_meter_provider(&resource, 30)?;
///     Ok(())
/// }
/// ```
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

/// Creates an environment filter for tracing based on the given level.
///
/// This function attempts to create a filter from environment variables first,
/// falling back to the provided level if no environment configuration is found.
///
/// # Arguments
///
/// * `level` - The default tracing level to use if no environment configuration is found
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::otel::init_env_filter;
/// use tracing::Level;
///
/// let filter = init_env_filter(&Level::INFO);
/// ```
pub fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}

/// Initializes the complete tracing stack with OpenTelemetry integration.
///
/// This function sets up the entire tracing infrastructure, including:
/// - OpenTelemetry tracing
/// - Metrics collection
/// - Log formatting
/// - Environment filtering
///
/// # Arguments
///
/// * `service_name` - The name of your service
/// * `attributes` - Additional key-value pairs to include in the resource
/// * `sample_ratio` - The ratio of traces to sample (0.0 to 1.0)
/// * `metrics_interval_secs` - The interval in seconds between metric collections
/// * `level` - The default tracing level
/// * `fmt_layer` - A formatting layer for the tracing output
///
/// # Returns
///
/// Returns a `Result` containing the configured `SdkTracerProvider` and `SdkMeterProvider`,
/// or an error if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::otel::setup_tracing;
/// use opentelemetry::KeyValue;
/// use tracing::Level;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (tracer_provider, meter_provider) = setup_tracing(
///         "my-service",
///         &[KeyValue::new("environment", "production")],
///         1.0,
///         30,
///         Level::INFO,
///         tracing_subscriber::fmt::layer(),
///     )?;
///
///     // Your application code here...
///
///     // Cleanup when done
///     tracer_provider.shutdown()?;
///     meter_provider.shutdown()?;
///     Ok(())
/// }
/// ```
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
