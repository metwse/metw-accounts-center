use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use std::env;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

/// OpenTelemetry tracing layer.
pub fn otel_layer<S: tracing::Subscriber + for<'span> LookupSpan<'span>>()
-> OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer> {
    let otlp_exporter_endpoint = env::var("OTEL_EXPORTER").unwrap_or_else(|_| {
        println!(
            "OTEL_EXPORTER environment variable is not set, defaulting to http://localhost:4317"
        );

        "http://localhost:4317".to_string()
    });

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_exporter_endpoint)
        .build()
        .unwrap();

    let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(env!("CARGO_CRATE_NAME"))
                .build(),
        )
        .build()
        .tracer("accounts-center");

    OpenTelemetryLayer::new(tracer)
}
