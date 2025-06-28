use axum::{
    extract::{ConnectInfo, MatchedPath},
    http,
};
use opentelemetry::trace::SpanKind;
use std::net::SocketAddr;
use tower_http::trace::MakeSpan;
use tracing::{field::Empty, Level};
use tracing_otel_extra::{
    dyn_span,
    extract::{context, fields},
};

/// An implementor of [`MakeSpan`] which creates `tracing` spans populated with information about
/// the request received by an `axum` web server.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/make_span.rs).
///
/// This span creator automatically adds the following attributes to each span:
///
/// - `http.method`: The HTTP method
/// - `http.route`: The matched route
/// - `http.client_ip`: The client's IP address
/// - `http.host`: The Host header
/// - `http.user_agent`: The User-Agent header
/// - `request_id`: A unique request identifier
/// - `trace_id`: The OpenTelemetry trace ID
///
/// # Example
///
/// ```rust
/// use axum_otel::{AxumOtelSpanCreator, Level};
/// use tower_http::trace::TraceLayer;
///
/// let layer = TraceLayer::new_for_http()
///     .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelSpanCreator {
    level: Level,
}

impl AxumOtelSpanCreator {
    /// Create a new `AxumOtelSpanCreator`.
    pub fn new() -> Self {
        Self {
            level: Level::TRACE,
        }
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Defaults to [`Level::TRACE`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl Default for AxumOtelSpanCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> MakeSpan<B> for AxumOtelSpanCreator {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let http_method = request.method().as_str();
        let http_route = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str());

        let client_ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(ip)| tracing::field::debug(ip));

        let span_name = http_route.as_ref().map_or_else(
            || http_method.to_string(),
            |route| format!("{http_method} {route}"),
        );

        let span = dyn_span!(
            self.level,
            "request",
            http.client_ip = client_ip,
            http.versions = ?request.version(),
            http.host = ?fields::extract_host(request),
            http.method = ?fields::extract_http_method(request),
            http.route = http_route,
            http.scheme = ?fields::extract_http_scheme(request),
            http.status_code = Empty,
            http.target = request.uri().path_and_query().map(|p| p.as_str()),
            http.user_agent = ?fields::extract_user_agent(request),
            otel.name = span_name,
            otel.kind = ?SpanKind::Server,
            otel.status_code = Empty,
            request_id = %fields::extract_request_id(request),
            trace_id = Empty
        );
        context::set_otel_parent(request.headers(), &span);
        span
    }
}
