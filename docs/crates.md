# Crates

The repository is a workspace, not one all-inclusive package.

| Crate | Responsibility | Use it when |
| --- | --- | --- |
| `otel-init` | Resources, OTLP tracer/meter/logger providers, subscriber setup, and provider shutdown | You own the subscriber or only need provider-level initialization |
| `tracing-otel` | HTTP fields/context/span helpers, dynamic tracing macros, and optional application logging bootstrap | You need shared tracing behavior or the opinionated `Logger` facade |
| `axum-otel` | Axum-specific `TraceLayer` callbacks | You need inbound Axum request spans |

The dependency direction is:

```text
axum-otel -> tracing-otel -> otel-init
```

These arrows are feature-dependent. For example, `tracing-otel` does not pull
in `otel-init` when a consumer enables only `fields`, and it has no default
features.

## Public entry points

### `otel-init`

- `get_resource`
- `init_tracer_provider`, `init_meter_provider`, `init_logger_provider`
- `OtelGuard`
- `init_env_filter` and `init_tracing_subscriber` with the `subscriber` feature

### `tracing-otel`

- `http::fields`, `http::propagation`, `http::context`, and `http::span`
- `dyn_span!` and `dyn_event!`
- `Logger`, `LoggerGuard`, `LogFormat`, and file-appender configuration
- `Logger::from_env` and environment initialization helpers with `env`

### `axum-otel`

- `AxumOtelSpanCreator`
- `AxumOtelOnResponse`
- `AxumOtelOnFailure`

Use docs.rs as the exhaustive API reference; this book documents selection,
composition, and runtime behavior.
