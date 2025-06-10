use anyhow::{anyhow, Result};
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::Layer,
    registry::Registry,
    EnvFilter,
};

// Define an enumeration for log formats
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub enum LogFormat {
    #[serde(rename = "compact")]
    #[default]
    Compact,
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "json")]
    Json,
}

// Parse log format from string
#[allow(dead_code)]
pub(crate) fn parse_log_format(s: &str) -> Result<LogFormat> {
    match s.to_lowercase().as_str().trim() {
        "compact" => Ok(LogFormat::Compact),
        "pretty" => Ok(LogFormat::Pretty),
        "json" => Ok(LogFormat::Json),
        _ => Err(anyhow!("Invalid log format: {}", s)),
    }
}

// Parse attributes from string
#[allow(dead_code)]
pub(crate) fn parse_attributes(s: &str) -> Result<Vec<KeyValue>> {
    s.trim()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let (key, value) = s
                .split_once('=')
                .ok_or_else(|| anyhow!("Invalid attribute: '{}'", s.trim()))?;

            let key = key.trim();
            let value = value.trim();

            if key.is_empty() || value.is_empty() {
                return Err(anyhow!("Empty key or value: '{}'", s.trim()));
            }

            Ok(KeyValue::new(key.to_string(), value.to_string()))
        })
        .collect()
}

// Initialize format layer
pub fn init_format_layer(format: LogFormat, ansi: bool) -> Box<dyn Layer<Registry> + Sync + Send> {
    let layer = fmt::Layer::default()
        .with_ansi(ansi)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let layer: Box<dyn Layer<Registry> + Sync + Send> = match format {
        LogFormat::Compact => layer.compact().boxed(),
        LogFormat::Pretty => layer.pretty().boxed(),
        LogFormat::Json => {
            let fmt_format = fmt::format().json().flatten_event(true);
            let json_fields = fmt::format::JsonFields::new();
            layer
                .event_format(fmt_format)
                .fmt_fields(json_fields)
                .boxed()
        }
    };
    layer
}

// Initialize env filter from level
pub(crate) fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_format() {
        assert_eq!(parse_log_format("compact").unwrap(), LogFormat::Compact);
        assert_eq!(parse_log_format("pretty").unwrap(), LogFormat::Pretty);
        assert_eq!(parse_log_format("json").unwrap(), LogFormat::Json);
    }

    #[test]
    fn test_parse_attributes() {
        // Test empty string
        assert_eq!(parse_attributes("").unwrap(), vec![]);
        assert_eq!(parse_attributes("   ").unwrap(), vec![]);

        // Test valid attributes
        let attrs = parse_attributes("key1=value1,key2=value2").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key.as_str(), "key1");
        assert_eq!(attrs[0].value.as_str(), "value1");
        assert_eq!(attrs[1].key.as_str(), "key2");
        assert_eq!(attrs[1].value.as_str(), "value2");

        // Test attributes with spaces
        let attrs = parse_attributes(" key1 = value1 , key2 = value2 ").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key.as_str(), "key1");
        assert_eq!(attrs[0].value.as_str(), "value1");
        assert_eq!(attrs[1].key.as_str(), "key2");
        assert_eq!(attrs[1].value.as_str(), "value2");

        // Test invalid formats
        assert!(parse_attributes("key1").is_err());
        assert!(parse_attributes("key1=").is_err());
        assert!(parse_attributes("=value1").is_err());
        assert!(parse_attributes("key1=value1,invalid").is_err());
    }
}
