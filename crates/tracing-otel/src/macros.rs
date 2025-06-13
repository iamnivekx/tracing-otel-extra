/// A macro to emit tracing events with dynamic level.
///
/// This macro is used to emit tracing event and span with dynamic level.
/// https://github.com/tokio-rs/tracing/issues/2730
///
/// # Example
///
/// ```rust
/// use tracing_otel_extra::{dyn_event};
/// use tracing::Level;
///
/// let level = Level::INFO;
/// dyn_event!(level, "request");
/// ```
#[macro_export]
macro_rules! dyn_event {
    ($lvl:expr, $($tt:tt)*) => {
        match $lvl {
            tracing::Level::ERROR => tracing::event!(tracing::Level::ERROR, $($tt)*),
            tracing::Level::WARN => tracing::event!(tracing::Level::WARN, $($tt)*),
            tracing::Level::INFO => tracing::event!(tracing::Level::INFO, $($tt)*),
            tracing::Level::DEBUG => tracing::event!(tracing::Level::DEBUG, $($tt)*),
            tracing::Level::TRACE => tracing::event!(tracing::Level::TRACE, $($tt)*),
        }
    };
}

#[macro_export]
macro_rules! dyn_span {
    ($lvl:expr, $($tt:tt)*) => {
        match $lvl {
            tracing::Level::ERROR => tracing::span!(tracing::Level::ERROR, $($tt)*),
            tracing::Level::WARN => tracing::span!(tracing::Level::WARN, $($tt)*),
            tracing::Level::INFO => tracing::span!(tracing::Level::INFO, $($tt)*),
            tracing::Level::DEBUG => tracing::span!(tracing::Level::DEBUG, $($tt)*),
            tracing::Level::TRACE => tracing::span!(tracing::Level::TRACE, $($tt)*),
        }
    };
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use tracing::Level;

    #[test]
    fn test_basic_usage() {
        let level = Level::INFO;
        // Test all log levels
        dyn_event!(level, "error message");
        dyn_event!(level, "warning message");
        dyn_event!(level, "info message");
        dyn_event!(level, "debug message");
        dyn_event!(level, "trace message");
    }

    #[test]
    fn test_with_fields() {
        let level = Level::INFO;
        dyn_event!(level, field1 = "value1", field2 = 42, "message with fields");
    }

    #[test]
    fn test_dyn_span() {
        let level = Level::INFO;
        dyn_span!(level, "span message");
    }
}
