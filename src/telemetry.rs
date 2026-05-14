use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::error::AppError;

pub fn init(rust_log: &str) -> Result<(), AppError> {
    let resource = Resource::new([KeyValue::new("service.name", "visdom-harness")]);

    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer("visdom-harness");
    global::set_tracer_provider(provider);

    let otel_layer = OpenTelemetryLayer::new(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer().json();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(rust_log));

    Registry::default()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(())
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
