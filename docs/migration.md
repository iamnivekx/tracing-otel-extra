# Migrating from 0.33.0

Version `0.33.1` introduced new crates.io package names. Cargo package names do
not migrate automatically.

```toml
# Before
tracing-otel-extra = "0.33.0"
tracing-opentelemetry-extra = "0.33.0"

# After
tracing-otel = "0.33.1"
otel-init = "0.33.1"
```

Update Rust import paths at the same time:

```rust,ignore
// Before
use tracing_otel_extra::Logger;
use tracing_opentelemetry_extra::OtelGuard;

// After
use tracing_otel::Logger;
use otel_init::OtelGuard;
```

`axum-otel` keeps its package and import name, but version `0.33.1` depends on
`tracing-otel` internally.

Also verify explicit `tracing-otel` features. The crate has no default
features: use `logger` for code-based `Logger`, `env` for `Logger::from_env`, or
the narrower HTTP features listed in [Cargo features](features.md).

If the application is upgrading from an older HTTP-span release, migrate
dashboard and alert field names using the table in
[HTTP span attributes](http-spans.md).
