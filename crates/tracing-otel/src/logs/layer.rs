use anyhow::Result;
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::Layer,
    registry::Registry,
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
pub fn deserialize_log_format<'de, D>(deserializer: D) -> Result<LogFormat, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str().trim() {
        "compact" => Ok(LogFormat::Compact),
        "pretty" => Ok(LogFormat::Pretty),
        "json" => Ok(LogFormat::Json),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid log format: '{}'",
            s
        ))),
    }
}

// Parse attributes from string
pub fn deserialize_attributes<'de, D>(deserializer: D) -> Result<Vec<KeyValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(Vec::new());
    }

    s.split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let s = s.trim();
            let (key, value) = s
                .split_once('=')
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid attribute: '{}'", s)))?;

            let key = key.trim();
            let value = value.trim();

            if key.is_empty() || value.is_empty() {
                return Err(serde::de::Error::custom(format!(
                    "Empty key or value: '{}'",
                    s
                )));
            }

            Ok(KeyValue::new(key.to_string(), value.to_string()))
        })
        .collect()
}

pub fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

// Configure layer based on log format
pub fn configure_log_format(
    layer: fmt::Layer<Registry>,
    format: LogFormat,
) -> Box<dyn Layer<Registry> + Send + Sync> {
    match format {
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
    }
}

// Initialize format layer
pub fn init_format_layer(
    format: LogFormat,
    ansi: bool,
    span_events: FmtSpan,
) -> Box<dyn Layer<Registry> + Sync + Send> {
    let layer = fmt::Layer::default()
        .with_ansi(ansi)
        .with_span_events(span_events);

    configure_log_format(layer, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    type StrDeserializer<'a> = serde::de::value::StrDeserializer<'a, serde::de::value::Error>;

    #[test]
    fn test_parse_log_format() {
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("compact".into_deserializer()).unwrap(),
            LogFormat::Compact
        );
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("pretty".into_deserializer()).unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(
            deserialize_log_format::<StrDeserializer>("json".into_deserializer()).unwrap(),
            LogFormat::Json
        );

        assert_eq!(
            deserialize_log_format::<StrDeserializer>("default_string".into_deserializer())
                .unwrap_err()
                .to_string(),
            "Invalid log format: 'default_string'"
        );
    }

    #[test]
    fn test_parse_attributes() {
        assert_eq!(
            deserialize_attributes::<StrDeserializer>("".into_deserializer()).unwrap(),
            vec![]
        );

        assert_eq!(
            deserialize_attributes::<StrDeserializer>(" ".into_deserializer()).unwrap(),
            vec![]
        );

        // Test valid attributes
        let attrs = deserialize_attributes::<StrDeserializer>(
            "key1=value1,key2=value2".into_deserializer(),
        )
        .unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key.as_str(), "key1");
        assert_eq!(attrs[0].value.as_str(), "value1");
        assert_eq!(attrs[1].key.as_str(), "key2");
        assert_eq!(attrs[1].value.as_str(), "value2");

        // Test attributes with spaces
        let attrs = deserialize_attributes::<StrDeserializer>(
            " key1 = value1 , key2 = value2 ".into_deserializer(),
        )
        .unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key.as_str(), "key1");
        assert_eq!(attrs[0].value.as_str(), "value1");
        assert_eq!(attrs[1].key.as_str(), "key2");
        assert_eq!(attrs[1].value.as_str(), "value2");

        // Test invalid formats
        assert!(deserialize_attributes::<StrDeserializer>("key1".into_deserializer()).is_err());
        assert!(deserialize_attributes::<StrDeserializer>("key1=".into_deserializer()).is_err());
        assert!(deserialize_attributes::<StrDeserializer>("=value1".into_deserializer()).is_err());
        assert!(deserialize_attributes::<StrDeserializer>(
            "key1=value1,invalid".into_deserializer()
        )
        .is_err());
    }
}
