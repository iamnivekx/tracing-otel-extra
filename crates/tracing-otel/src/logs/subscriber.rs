use crate::otel::{get_resource, init_meter_provider, init_tracer_provider};
use anyhow::Result;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

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
/// use tracing_otel_extra::logs::init_env_filter;
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
/// use tracing_otel_extra::logs::setup_tracing;
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
    use opentelemetry::trace::TracerProvider as _;

    // Build resource with service name and additional attributes
    let resource = get_resource(service_name, attributes);
    let tracer_provider = init_tracer_provider(&resource, sample_ratio)?;
    let meter_provider = init_meter_provider(&resource, metrics_interval_secs)?;
    // let logger_provider = init_logger_provider(&resource)?;

    // Set up env filter
    let env_filter = init_env_filter(&level);

    // Set up telemetry layer with tracer
    let tracer = tracer_provider.tracer(service_name.to_string());
    let metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(metrics_layer)
        .with(otel_layer)
        .with(env_filter)
        .init();

    Ok((tracer_provider, meter_provider))
}
