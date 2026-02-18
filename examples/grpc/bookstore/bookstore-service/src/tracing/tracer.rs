use std::error::Error;

use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_stdout::SpanExporter;
use tracing_subscriber::{EnvFilter, fmt::format::Format, prelude::*};

use crate::config::AppConfig;

/// Tracer configuration and initialization.
///
/// Handles setting up distributed tracing for the bookstore service.
pub struct Tracer;

impl Tracer {
    /// Installs stdout tracing with OpenTelemetry.
    ///
    /// # Errors
    ///
    /// Returns an error if tracer initialization fails.
    pub fn install_stdout() -> Result<(), Box<dyn Error + Send + Sync>> {
        let config = AppConfig::get();

        global::set_text_map_propagator(TraceContextPropagator::new());

        let mut resource = Resource::builder().with_service_name(config.distribution.name.clone());
        if let Some(version) = config.distribution.version.clone() {
            resource = resource.with_attribute(KeyValue::new("version", version));
        }

        let provider = SdkTracerProvider::builder()
            .with_resource(resource.build())
            .with_simple_exporter(SpanExporter::default())
            .build();
        global::set_tracer_provider(provider);

        let layer = tracing_subscriber::fmt::layer()
            .event_format(Format::default().pretty())
            .with_filter(EnvFilter::from_default_env());

        tracing_subscriber::registry().with(layer).init();

        Ok(())
    }
}
