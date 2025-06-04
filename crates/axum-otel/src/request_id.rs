use axum::http::{HeaderMap, HeaderName};
use tracing::{field, Span};

pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub const REQUEST_ID: HeaderName = HeaderName::from_static("request-id");

/// Get the request id from the headers. `x-request-id` or `request-id` header is supported.
pub fn get_request_id(headers: &HeaderMap) -> &str {
    headers
        .get(X_REQUEST_ID)
        .or_else(|| headers.get(REQUEST_ID))
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use tracing::Level;

    #[test]
    fn test_get_request_id_with_x_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REQUEST_ID, "test-id-1".parse().unwrap());
        let request_id = get_request_id(&headers);
        assert_eq!(request_id, "test-id-1");
    }

    #[test]
    fn test_get_request_id_without_request_id() {
        let headers = HeaderMap::new();
        let request_id = get_request_id(&headers);
        assert_eq!(request_id, "");
    }

    #[test]
    fn test_get_request_id_with_x_request_id_and_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REQUEST_ID, "test-id-1".parse().unwrap());
        headers.insert(REQUEST_ID, "test-id-2".parse().unwrap());
        let request_id = get_request_id(&headers);
        assert_eq!(request_id, "test-id-1");
    }
}
