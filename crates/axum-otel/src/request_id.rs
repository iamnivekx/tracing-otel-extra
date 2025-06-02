use axum::http::HeaderMap;
use tracing::Span;

pub(crate) fn set_request_id(headers: &HeaderMap, span: &Span) {
    let request_id = headers
        .get("x-request-id")
        .and_then(|id| id.to_str().map(ToOwned::to_owned).ok())
        // If `x-request-id` isn't set, check `request_id`.
        .or_else(|| {
            headers
                .get("request_id")
                .and_then(|v| v.to_str().map(ToOwned::to_owned).ok())
        })
        .unwrap_or_else(|| "".to_string());
    span.record("request_id", tracing::field::display(request_id));
}
