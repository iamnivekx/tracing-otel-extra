# otel-kit

`otel-kit` is the repository for three Rust crates that cover
different observability boundaries:

- `otel-init` initializes OpenTelemetry providers and tracing subscribers;
- `tracing-otel` provides shared HTTP tracing utilities and an opinionated
logging/bootstrap facade;
- `axum-otel` adapts the shared HTTP span behavior to Axum and
`tower-http::TraceLayer`.

Choose the smallest crate that owns the behavior your application needs. An
Axum application can use `axum-otel` with its own subscriber, while an
application that wants the repository's complete logging setup can combine it
with `tracing-otel`'s `logger` or `env` feature. Provider-only users can depend
directly on `otel-init`.

The workspace packages are currently version `0.33.1`. `Cargo.toml` declares
Rust 1.96.0 as the minimum supported Rust version. The repository toolchain may
pin a newer compiler for local development.

## Where to start

- [Getting started](getting-started.md) — install the right crates and build a
minimal application.
- [Crates](crates.md) — choose the smallest crate for each observability
responsibility.
- [Cargo features](features.md) — keep optional dependencies and capabilities
explicit.
- [Comparison with](comparison.md) `axum-tracing-opentelemetry` — choose between
focused request middleware and this workspace's broader observability setup.
- [OTLP export and lifecycle](otlp.md) — configure exporters and shut providers
down safely.
- [HTTP span attributes](http-spans.md) — understand emitted fields before
writing dashboards or alerts.

The complete type-level references are on
[docs.rs for](https://docs.rs/tracing-otel) `tracing-otel`,
[docs.rs for](https://docs.rs/otel-init) `otel-init`, and
[docs.rs for](https://docs.rs/axum-otel) `axum-otel`.