# Examples

The workspace contains runnable examples rather than embedding a second copy of
their source in this book.

## Basic Axum service

`examples/otel` initializes `Logger` from `LOG_*`, adds request ID middleware,
and composes the three `axum-otel` callbacks.

```sh
cargo run -p axum-otel-demo
```

Without an OTLP endpoint, providers stay local-only. To export all supported
signals to a local collector:

```sh
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
cargo run -p axum-otel-demo
```

The service listens on `0.0.0.0:8080`; call `/hello` for an instrumented route
or `/health` for the simple health route.

## Microservices demonstration

`examples/microservices` contains users and articles services, outbound
Reqwest tracing, request ID propagation, and a Grafana/Loki/Tempo-oriented
Docker setup. Follow its README and architecture guide when validating full
cross-service propagation.

These examples are integration demonstrations, not additional public APIs.
Use the crate-specific docs.rs pages for type and method details.
