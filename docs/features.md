# Cargo features

`tracing-otel` has no default features. Every application must select the
modules its source code uses.

| Feature | Adds | Implies |
| --- | --- | --- |
| `fields` | HTTP request field extraction | `http` dependency only |
| `macros` | Runtime-level `dyn_span!` and `dyn_event!` | `tracing` dependency |
| `http` | HTTP context extraction and injection | `fields` |
| `context` | Current trace/span IDs and remote-parent attachment | `http` |
| `span` | Shared HTTP server span creation | `context`, `macros` |
| `otel` | Re-exports from `otel-init` | `otel-init` dependency |
| `logger` | Opinionated subscriber, console/file logging, traces, metrics, and optional OTLP logs | `otel`, `otel-init/subscriber` |
| `env` | Environment deserialization for `Logger` | `logger` |

Examples:

```toml
# Shared request-span utilities without the Logger facade.
tracing-otel = { version = "0.33.1", features = ["span"] }

# Complete code-configured application bootstrap.
tracing-otel = { version = "0.33.1", features = ["logger"] }

# Logger plus environment configuration.
tracing-otel = { version = "0.33.1", features = ["env"] }
```

`axum-otel` enables the `tracing-otel` capabilities it uses internally. A
consumer does not need to duplicate those internal feature choices unless it
also imports `tracing-otel` APIs directly.

`otel-init` has one optional feature:

| Feature | Adds |
| --- | --- |
| `subscriber` | `tracing-subscriber` integration and the OpenTelemetry log bridge |

Useful contributor checks:

```sh
cargo check -p tracing-otel --no-default-features
cargo check -p tracing-otel --no-default-features --features span
cargo check -p tracing-otel --no-default-features --features logger
cargo check -p tracing-otel --no-default-features --features env
```
