/// A macro to emit tracing events with dynamic level.
///
/// This macro is used to emit tracing events with dynamic level.
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/635692d757f29dfa3041c02cd66c195be07bc8b3/tower-http/src/trace/mod.rs#L414).
///
/// # Example
///
/// ```rust
/// use axum_otel::{event_dynamic_lvl, Level};
///
/// event_dynamic_lvl!(Level::INFO, "request");
/// ```
#[macro_export]
macro_rules! event_dynamic_lvl {
    ($lvl:expr, $($tt:tt)*) => {
        match $lvl {
            tracing::Level::ERROR => {
                tracing::event!(tracing::Level::ERROR, $($tt)*);
            }
            tracing::Level::WARN => {
                tracing::event!(tracing::Level::WARN, $($tt)*);
            }
            tracing::Level::INFO => {
                tracing::event!(tracing::Level::INFO, $($tt)*);
            }
            tracing::Level::DEBUG => {
                tracing::event!(tracing::Level::DEBUG, $($tt)*);
            }
            tracing::Level::TRACE => {
                tracing::event!(tracing::Level::TRACE, $($tt)*);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Level;

    #[test]
    fn test_basic_usage() {
        // Test all log levels
        event_dynamic_lvl!(Level::ERROR, "error message");
        event_dynamic_lvl!(Level::WARN, "warning message");
        event_dynamic_lvl!(Level::INFO, "info message");
        event_dynamic_lvl!(Level::DEBUG, "debug message");
        event_dynamic_lvl!(Level::TRACE, "trace message");
    }

    #[test]
    fn test_with_fields() {
        event_dynamic_lvl!(
            Level::INFO,
            field1 = "value1",
            field2 = 42,
            "message with fields"
        );
    }
}
