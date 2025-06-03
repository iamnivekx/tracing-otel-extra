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
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
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
        .with_attribute(KeyValue::new(resource::SERVICE_NAME, "articles-service"))
        .build()
});

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Article {
    id: u64,
    title: String,
    content: String,
    author_id: u64,
}

#[derive(Debug, Deserialize)]
struct CreateArticle {
    title: String,
    content: String,
    author_id: u64,
}

#[derive(Clone, Debug)]
struct AppState {
    articles: Arc<tokio::sync::RwLock<Vec<Article>>>,
    http_client: ClientWithMiddleware,
}

#[tracing::instrument]
async fn get_articles(State(state): State<AppState>) -> Json<Vec<Article>> {
    let articles = state.articles.read().await;
    Json(articles.clone())
}

#[tracing::instrument]
async fn get_article(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Article>, (axum::http::StatusCode, String)> {
    let articles = state.articles.read().await;
    let article = articles.iter().find(|a| a.id == id);
    match article {
        Some(article) => Ok(Json(article.clone())),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            "Article not found".to_string(),
        )),
    }
}

#[tracing::instrument]
async fn get_articles_by_author(
    State(state): State<AppState>,
    Path(author_id): Path<u64>,
) -> Result<Json<Vec<Article>>, (axum::http::StatusCode, String)> {
    // First verify that the author exists by calling the users service
    let user_url = format!("http://localhost:8081/users/{}", author_id);
    match state.http_client.get(&user_url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return Err((
                    axum::http::StatusCode::NOT_FOUND,
                    format!("Author with id {} not found", author_id),
                ));
            }
        }
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to verify author: {}", e),
            ));
        }
    }

    // If we get here, the author exists, so we can return their articles
    let articles = state.articles.read().await;
    let author_articles: Vec<Article> = articles
        .iter()
        .filter(|a| a.author_id == author_id)
        .cloned()
        .collect();
    Ok(Json(author_articles))
}

#[tracing::instrument]
async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<CreateArticle>,
) -> Json<Article> {
    let mut articles = state.articles.write().await;
    let id = articles.len() as u64 + 1;
    let article = Article {
        id,
        title: payload.title,
        content: payload.content,
        author_id: payload.author_id,
    };
    articles.push(article.clone());
    Json(article)
}

fn init_telemetry() -> opentelemetry_sdk::trace::SdkTracerProvider {
    let (loki_layer, loki_task) = tracing_loki::builder()
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
    let tracer = provider.tracer(env!("CARGO_PKG_NAME"));

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

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let state = AppState {
        articles: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        http_client: client,
    };

    let app = Router::new()
        .route("/articles", get(get_articles))
        .route("/articles/{id}", get(get_article))
        .route("/articles/author/{author_id}", get(get_articles_by_author))
        .route("/articles", post(create_article))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator)
                        .on_response(AxumOtelOnResponse)
                        .on_failure(AxumOtelOnFailure),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8082").await?;
    tracing::info!("Articles service listening on 127.0.0.1:8082");
    axum::serve(listener, app.into_make_service()).await?;

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider.");

    Ok(())
}
