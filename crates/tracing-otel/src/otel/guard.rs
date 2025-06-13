use anyhow::Result;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};

/// A guard that holds the tracer provider and ensures proper cleanup
#[derive(Debug, Clone)]
pub struct ProviderGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}

impl ProviderGuard {
    /// Create a new guard with the given provider
    pub fn new(
        tracer_provider: Option<SdkTracerProvider>,
        meter_provider: Option<SdkMeterProvider>,
    ) -> Self {
        Self {
            tracer_provider,
            meter_provider,
        }
    }

    // Set the tracer provider
    pub fn with_tracer_provider(mut self, tracer_provider: SdkTracerProvider) -> Self {
        self.tracer_provider = Some(tracer_provider);
        self
    }

    // Set the meter provider
    pub fn with_meter_provider(mut self, meter_provider: SdkMeterProvider) -> Self {
        self.meter_provider = Some(meter_provider);
        self
    }

    /// Manually shutdown the tracer provider
    pub fn shutdown(mut self) -> Result<()> {
        if let Some(tracer_provider) = self.tracer_provider.take() {
            tracer_provider.shutdown()?;
        }
        if let Some(meter_provider) = self.meter_provider.take() {
            meter_provider.shutdown()?;
        }
        Ok(())
    }
}

// Drop the guard and shutdown the providers
impl Drop for ProviderGuard {
    fn drop(&mut self) {
        if let Some(tracer_provider) = self.tracer_provider.take() {
            if let Err(err) = tracer_provider.shutdown() {
                eprintln!("{err:?}");
            }
        }
        if let Some(meter_provider) = self.meter_provider.take() {
            if let Err(err) = meter_provider.shutdown() {
                eprintln!("{err:?}");
            }
        }
    }
}
