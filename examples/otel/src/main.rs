use anyhow::Result;
use axum::extract::Query;
use axum::{routing::get, Router};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_otel_extra::Logger;

#[derive(Debug, Deserialize, Serialize)]
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    // Consider changing to anyhow::Result for broader error handling
    let mut logger = Logger::from_env(Some("LOG_"))?;
    logger = logger.with_ansi(true);
    logger.init()?;

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
    info!("Server is running on http://127.0.0.1:8080");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
