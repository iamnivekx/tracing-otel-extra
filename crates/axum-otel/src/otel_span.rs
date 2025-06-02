//! Module defining utilities for crating `tracing` spans compatible with OpenTelemetry's
//! conventions.
use crate::{otel::set_otel_parent, request_id::set_request_id};
use axum::{
    extract::{ConnectInfo, MatchedPath},
    http,
};
use opentelemetry::trace::SpanKind;
use std::net::SocketAddr;
use tower_http::{
    classify::ServerErrorsFailureClass,
    trace::{MakeSpan, OnFailure, OnResponse},
};
use tracing::field::{Empty, debug};

/// An implementor of [`MakeSpan`] which creates `tracing` spans populated with information about
/// the request received by an `axum` web server.
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelSpanCreator;

impl<B> MakeSpan<B> for AxumOtelSpanCreator {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let http_method = request.method().as_str();
        let http_route = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str());

        let user_agent = request
            .headers()
            .get(http::header::USER_AGENT)
            .and_then(|header| header.to_str().ok());

        let host = request
            .headers()
            .get(http::header::HOST)
            .and_then(|header| header.to_str().ok());

        let client_ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(ip)| debug(ip));

        let span_name = http_route.as_ref().map_or_else(
            || http_method.to_string(),
            |route| format!("{} {}", http_method, route),
        );
        let span = tracing::error_span!(
            "request",
            http.client_ip = client_ip,
            http.versions = ?request.version(),
            http.host = host,
            http.method = ?request.method(),
            http.route = http_route,
            http.scheme = request.uri().scheme().map(debug),
            http.status_code = Empty,
            http.target = request.uri().path_and_query().map(|p| p.as_str()),
            http.user_agent = user_agent,
            otel.name = span_name,
            otel.kind = ?SpanKind::Server,
            otel.status_code = Empty,
            request_id = Empty,
            trace_id = Empty,
        );
        set_request_id(&request.headers(), &span);
        set_otel_parent(&request.headers(), &span);
        span
    }
}

/// An implementor of [`OnResponse`] which records the response status code and latency.
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelOnResponse;

impl<B> OnResponse<B> for AxumOtelOnResponse {
    fn on_response(
        self,
        response: &http::Response<B>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        let status = response.status().as_u16().to_string();
        span.record("http.status_code", tracing::field::display(status));
        span.record("otel.status_code", "OK");

        tracing::debug!(
            "finished processing request latency={} ms status={}",
            latency.as_millis(),
            response.status().as_u16(),
        );
    }
}

/// An implementor of [`OnFailure`] which records the failure status code.
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelOnFailure;

impl OnFailure<ServerErrorsFailureClass> for AxumOtelOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: ServerErrorsFailureClass,
        _latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        match failure_classification {
            ServerErrorsFailureClass::StatusCode(status) if status.is_server_error() => {
                span.record("otel.status_code", "ERROR");
            }
            _ => {}
        }
    }
}
