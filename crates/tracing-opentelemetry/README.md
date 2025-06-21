# tracing-opentelemetry-extra

**Reference:** This crate is mainly organized based on the [official tracing-opentelemetry OTLP example](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs).

This crate provides enhanced OpenTelemetry integration for tracing applications. It's based on the [tracing-opentelemetry examples](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs) and provides a clean, easy-to-use API for setting up OpenTelemetry tracing and metrics.

## Features

- Easy OpenTelemetry initialization with OTLP exporter
- Configurable sampling and resource attributes
- Automatic cleanup with guard pattern
- Support for both tracing and metrics
- Clean separation of concerns from other tracing libraries

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tracing-opentelemetry-extra = "0.30.x"
```

## Quick Start

### Basic Usage

```rust
use opentelemetry::KeyValue;
use tracing_opentelemetry_extra::{get_resource, init_tracer_provider, init_meter_provider, OtelGuard};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource with service name and attributes
    let resource = get_resource(
        "my-service",
        &[
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.0.0"),
        ],
    );

    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource, 1.0)?;
    let meter_provider = init_meter_provider(&resource, 30)?;

    // Create guard for automatic cleanup
    let _guard = OtelGuard::new(Some(tracer_provider), Some(meter_provider));

    // Your application code here...
    tracing::info!("Application started");

    // Cleanup is handled automatically when the guard is dropped
    Ok(())
}
```

### With Tracing Subscriber

```rust
use opentelemetry::KeyValue;
use tracing_opentelemetry_extra::{get_resource, init_tracer_provider, init_meter_provider, ProviderGuard};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource
    let resource = get_resource("my-service", &[KeyValue::new("environment", "production")]);
    
    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource, 1.0)?;
    let meter_provider = init_meter_provider(&resource, 30)?;

    // Set up tracing subscriber
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("my-service")))
        .with(tracing_opentelemetry::MetricsLayer::new(meter_provider))
        .with(env_filter)
        .init();

    // Create guard for cleanup
    let _guard = OtelGuard::new(Some(tracer_provider), Some(meter_provider));

    // Your application code here...
    tracing::info!("Application started with OpenTelemetry");

    Ok(())
}
```

## API Reference

### Core Functions

#### `get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource`

Creates an OpenTelemetry resource with the given service name and attributes.

```rust
let resource = get_resource(
    "my-service",
    &[KeyValue::new("environment", "production")],
);
```

#### `init_tracer_provider(resource: &Resource, sample_ratio: f64) -> Result<SdkTracerProvider>`

Initializes a tracer provider with OTLP exporter and configurable sampling.

```rust
let tracer_provider = init_tracer_provider(&resource, 1.0)?;
```

#### `init_meter_provider(resource: &Resource, metrics_interval_secs: u64) -> Result<SdkMeterProvider>`

Initializes a meter provider with periodic metric collection.

```rust
let meter_provider = init_meter_provider(&resource, 30)?;
```

### OtelGuard

A guard that ensures proper cleanup of OpenTelemetry resources when dropped.

```rust
let guard = OtelGuard::new(Some(tracer_provider), Some(meter_provider));
// Resources will be automatically cleaned up when guard is dropped
```

## Configuration

### Sampling

Control the ratio of traces to sample (0.0 to 1.0):

```rust
// Sample 50% of traces
let tracer_provider = init_tracer_provider(&resource, 0.5)?;

// Sample all traces
let tracer_provider = init_tracer_provider(&resource, 1.0)?;
```

### Metrics Collection

Configure the interval for metrics collection:

```rust
// Collect metrics every 60 seconds
let meter_provider = init_meter_provider(&resource, 60)?;
```

### Resource Attributes

Add custom attributes to your service:

```rust
let resource = get_resource(
    "my-service",
    &[
        KeyValue::new("environment", "production"),
        KeyValue::new("version", "1.0.0"),
        KeyValue::new("region", "us-west-2"),
    ],
);
```

## Features

- `subscriber` (default): Enables tracing-subscriber integration

## Examples

See the [examples directory](../../examples/) for more detailed usage examples.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Related Crates

- [tracing-otel-extra](../tracing-otel/) - HTTP, context, fields, and span utilities
- [axum-otel](../axum-otel/) - Axum web framework integration 