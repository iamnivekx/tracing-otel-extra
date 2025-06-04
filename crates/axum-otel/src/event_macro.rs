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
    ( $(target: $target:expr,)? $(parent: $parent:expr,)? $lvl:expr, $($tt:tt)* ) => {
        match $lvl {
            tracing::Level::ERROR => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::ERROR,
                    $($tt)*
                );
            }
            tracing::Level::WARN => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::WARN,
                    $($tt)*
                );
            }
            tracing::Level::INFO => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::INFO,
                    $($tt)*
                );
            }
            tracing::Level::DEBUG => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::DEBUG,
                    $($tt)*
                );
            }
            tracing::Level::TRACE => {
                tracing::event!(
                    $(target: $target,)?
                    $(parent: $parent,)?
                    tracing::Level::TRACE,
                    $($tt)*
                );
            }
        }
    };
}
