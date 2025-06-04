use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use opentelemetry_semantic_conventions::resource;
use serde::{Deserialize, Serialize};
use std::process;
use std::sync::{Arc, LazyLock};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use url::Url;

static RESOURCE: LazyLock<Resource> = LazyLock::new(|| {
    Resource::builder()
        .with_attribute(KeyValue::new(resource::SERVICE_NAME, "users-service"))
        .build()
});

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Clone, Debug)]
struct AppState {
    users: Arc<tokio::sync::RwLock<Vec<User>>>,
}

#[tracing::instrument]
async fn get_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = state.users.read().await;
    Json(users.clone())
}

#[tracing::instrument]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<User>, (axum::http::StatusCode, String)> {
    let users = state.users.read().await;
    let user = users.iter().find(|u| u.id == id);
    match user {
        Some(user) => Ok(Json(user.clone())),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            "User not found".to_string(),
        )),
    }
}

#[tracing::instrument]
async fn create_user(State(state): State<AppState>, Json(payload): Json<CreateUser>) -> Json<User> {
    let mut users = state.users.write().await;
    let id = users.len() as u64 + 1;
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
    };
    users.push(user.clone());
    Json(user)
}

fn init_telemetry() -> opentelemetry_sdk::trace::SdkTracerProvider {
    let (loki_layer, loki_task) = tracing_loki::builder()
        .extra_field("service_name", env!("CARGO_PKG_NAME"))
        .expect("Failed to add service name field")
        .extra_field("pid", format!("{}", process::id()))
        .expect("Failed to add pid field")
        .build_url(Url::parse("http://127.0.0.1:3100").expect("Failed to parse URL"))
        .expect("Failed to build Loki layer");

    global::set_text_map_propagator(TraceContextPropagator::new());
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .expect("Failed to build the span exporter");
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(RESOURCE.clone())
        .build();
    let tracer = provider.tracer("users-service");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
            "{}=trace,tower_http=debug,axum::rejection=trace,axum_otel=trace",
            env!("CARGO_PKG_NAME")
        )
        .into()
    });
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_level(true);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(formatting_layer)
        .with(loki_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");

    tokio::spawn(loki_task);
    provider
}

#[tokio::main]
async fn main() -> Result<()> {
    let provider = init_telemetry();

    let state = AppState {
        users: Arc::new(tokio::sync::RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new())
                        .on_response(AxumOtelOnResponse::new())
                        .on_failure(AxumOtelOnFailure::new()),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8081").await?;
    tracing::info!("Users service listening on 127.0.0.1:8081");
    axum::serve(listener, app.into_make_service()).await?;

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider.");

    Ok(())
}
