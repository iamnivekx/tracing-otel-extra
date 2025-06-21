use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Creates a resource with the given service name and attributes.
///
/// This function builds an OpenTelemetry resource that identifies your service
/// and includes any additional attributes you want to track.
///
/// # Arguments
///
/// * `service_name` - The name of your service
/// * `attributes` - Additional key-value pairs to include in the resource
///
/// # Examples
///
/// ```rust
/// use tracing_opentelemetry_extra::get_resource;
/// use opentelemetry::KeyValue;
///
/// let resource = get_resource(
///     "my-service",
///     &[KeyValue::new("environment", "production")],
/// );
/// ```
pub fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(attributes.to_vec())
        .build()
}
