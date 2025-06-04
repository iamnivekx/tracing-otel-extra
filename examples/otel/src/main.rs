use anyhow::Result;
use axum::extract::Query;
use axum::{routing::get, Router};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use opentelemetry_semantic_conventions::resource;
use serde::Deserialize;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry}; // For Axum server

static RESOURCE: LazyLock<Resource> = LazyLock::new(|| {
    Resource::builder()
        .with_attribute(KeyValue::new(
            resource::SERVICE_NAME,
            env!("CARGO_CRATE_NAME"),
        ))
        .build()
});

#[derive(Deserialize, Debug)]
pub struct HelloQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[tracing::instrument]
async fn hello(q: Query<HelloQuery>) -> &'static str {
    debug!("hello request query: {:?}", q);
    "Hello world!"
}

#[tracing::instrument]
async fn health() -> &'static str {
    "OK"
}

fn init_telemetry() -> opentelemetry_sdk::trace::SdkTracerProvider {
    // Start a new otlp trace pipeline.
    // Spans are exported in batch - recommended setup for a production application.
    global::set_text_map_propagator(TraceContextPropagator::new());
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317") // Ensure OTel collector is running at this address
        .build()
        .expect("Failed to build the span exporter");
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(RESOURCE.clone())
        .build();
    let tracer = provider.tracer(env!("CARGO_CRATE_NAME"));

    // Filter based on level - trace, debug, info, warn, error
    // Tunable via `RUST_LOG` env variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // axum logs rejections from built-in extractors with the `axum::rejection`
        // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
        format!(
            "{}=trace,axum::rejection=trace,axum_otel=trace",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });
    // Create a `tracing` layer using the otlp tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // Create a `tracing` layer to emit spans as structured logs to stdout
    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_level(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    // Combined them all together in a `tracing` subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");

    provider
}

#[tokio::main]
async fn main() -> Result<()> {
    // Consider changing to anyhow::Result for broader error handling
    let provider = init_telemetry();

    // Setup Axum router and server
    let app = Router::new()
        .route("/hello", get(hello))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                        .on_failure(AxumOtelOnFailure::new().level(Level::ERROR)),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .route("/health", get(health)); // without request id, the span will not be created

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app.into_make_service()).await?;

    // Ensure all spans have been shipped.
    // In a real application, this might be part of a more graceful shutdown sequence.
    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider."); // Use .expect for more context on panic

    Ok(())
}
