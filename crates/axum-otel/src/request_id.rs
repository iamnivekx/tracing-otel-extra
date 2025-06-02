use axum::http::HeaderMap;
use tracing::Span;

/// Set the request id for the current span. `x-request-id` or `request-id` header is supported.
pub(crate) fn set_request_id(headers: &HeaderMap, span: &Span) {
    let request_id = headers
        .get("x-request-id")
        .and_then(|id| id.to_str().map(ToOwned::to_owned).ok())
        // If `x-request-id` isn't set, check `request_id`.
        .or_else(|| {
            headers
                .get("request-id")
                .and_then(|v| v.to_str().map(ToOwned::to_owned).ok())
        })
        .unwrap_or_default();
    span.record("request_id", tracing::field::display(request_id));
}
