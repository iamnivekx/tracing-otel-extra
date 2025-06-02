use axum::http::HeaderMap;

/// Set the parent span for the current span and record the trace id.
pub(crate) fn set_otel_parent(headers: &HeaderMap, span: &tracing::Span) {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;

    let remote_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&opentelemetry_http::HeaderExtractor(headers))
    });

    // If we have a remote parent span, this will be the parent's trace identifier.
    // If not, it will be the newly generated trace identifier with this request as root span.
    let remote_span = remote_context.span();
    let remote_span_context = remote_span.span_context();
    let trace_id = remote_span_context
        .is_valid()
        .then(|| remote_span_context.trace_id().to_string())
        .unwrap_or_else(|| span.context().span().span_context().trace_id().to_string());
    span.set_parent(remote_context);
    span.record("trace_id", tracing::field::display(trace_id));
}
