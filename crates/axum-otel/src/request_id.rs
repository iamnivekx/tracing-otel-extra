use axum::http::{HeaderMap, HeaderName};
use tracing::{field, Span};

pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub const REQUEST_ID: HeaderName = HeaderName::from_static("request-id");
pub const REQUEST_ID_FIELD: &str = "request_id";

/// Set the request id for the current span. `x-request-id` or `request-id` header is supported.
pub(crate) fn set_request_id(headers: &HeaderMap, span: &Span) {
    let request_id = headers
        .get(X_REQUEST_ID)
        .or_else(|| headers.get(REQUEST_ID))
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    span.record(REQUEST_ID_FIELD, field::display(request_id));
}
