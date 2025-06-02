# axum-otel

A structured logging middleware for Axum web framework that integrates with OpenTelemetry.

## Features

- Structured logging middleware for Axum
- OpenTelemetry integration
- Request tracing
- Metrics collection
- Customizable span attributes

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
axum-otel = "0.29.0"
axum = { version = "0.8", features = ["macros"] }
tower-http = { version = "0.6.5", features = ["trace"] }
opentelemetry = { version = "0.29.0", features = ["metrics"] }
opentelemetry_sdk = { version = "0.29.0", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.29.0", features = ["metrics", "grpc-tonic"] }
```

## Quick Start

```rust
use axum::{
    routing::get,
    Router,
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
use opentelemetry::sdk::trace::Config;
use opentelemetry_otlp::WithExportConfig;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initialize OpenTelemetry
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://localhost:4317"))
        .with_trace_config(Config::default())
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Failed to initialize OpenTelemetry");

    // Build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(AxumOtelSpanCreator)
                .on_response(AxumOtelOnResponse)
                .on_failure(AxumOtelOnFailure),
        );

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
```

## Examples

Check out the [examples](./examples) directory for more usage examples:

- [Basic OpenTelemetry integration](./examples/otel)

## Documentation

For more detailed documentation, visit [docs.rs](https://docs.rs/axum-otel/).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 