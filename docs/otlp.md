# OTLP export and lifecycle

`otel-init` creates trace, metric, and log providers. Export is opt-in through
an endpoint environment variable.

```sh
# Enable all configured signals.
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
```

Signal-specific endpoints can enable only one signal or override the shared
endpoint for that signal:

```sh
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces
export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf
```

The corresponding metric and log names are
`OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` / `_PROTOCOL` and
`OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` / `_PROTOCOL`. Supported protocol values in
this workspace are `grpc`, `http/protobuf` (also `http/proto`), and
`http/json`. A signal-specific protocol takes precedence over
`OTEL_EXPORTER_OTLP_PROTOCOL`; the default is `grpc`.

When neither the signal-specific endpoint nor the shared endpoint contains a
non-blank value, that provider remains local-only. It can still create trace
IDs, but it does not construct an exporter or attempt a connection to
`localhost:4317`.

`Logger` always initializes trace and metric providers. It initializes the log
provider only when `otel_logs_enabled` is true, for example with
`LOG_OTEL_LOGS_ENABLED=true` through the `env` feature.

## Shutdown order

Keep `LoggerGuard` or `OtelGuard` alive until application shutdown. Explicit
shutdown is available when the application needs to surface provider shutdown
errors:

```rust,no_run
# use tracing_otel::Logger;
# fn run() -> anyhow::Result<()> {
let guard = Logger::new("worker").init()?;
tracing::info!("draining work");
guard.shutdown()?;
# Ok(())
# }
```

`LoggerGuard` shuts down telemetry providers before releasing its optional
non-blocking file writer guard. Dropping a guard also performs cleanup, but an
explicit `shutdown()` is the path that returns provider shutdown errors.
