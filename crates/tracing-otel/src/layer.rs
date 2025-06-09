use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    layer::Layer,
    registry::Registry,
    EnvFilter,
};

// Define an enumeration for log formats
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub enum LogFormat {
    #[serde(rename = "compact")]
    #[default]
    Compact,
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "json")]
    Json,
}

// Initialize format layer
pub fn init_format_layer<W2>(
    make_writer: W2,
    format: LogFormat,
    ansi: bool,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    let layer = fmt::Layer::default()
        .with_ansi(ansi)
        .with_writer(make_writer);
    let layer: Box<dyn Layer<Registry> + Sync + Send> = match format {
        LogFormat::Compact => layer.compact().boxed(),
        LogFormat::Pretty => layer.pretty().boxed(),
        LogFormat::Json => layer.json().boxed(),
    };
    layer
}

// Initialize env filter from level
pub(crate) fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}
