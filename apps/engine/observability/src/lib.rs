//! Telemetry export: tracing subscriber, optional OTLP spans, and Prometheus metrics.

pub mod config;
pub mod prometheus;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub use config::{ObservabilityConfig, OtlpConfig};

/// Holds exporters alive; dropping this flushes pending telemetry.
#[must_use = "dropping the guard shuts down telemetry export"]
pub struct ObservabilityGuard {
    // Held here (not dropped immediately) because shutdown is what flushes the last batch.
    #[cfg(feature = "otel")]
    otel: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
}

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        #[cfg(feature = "otel")]
        if let Some(provider) = self.otel.take() {
            if let Err(error) = provider.shutdown() {
                tracing::error!(
                    target: "piramid::observability",
                    %error,
                    "OTLP exporter failed to flush on shutdown"
                );
            }
        }
    }
}

/// Installs the tracing subscriber and any configured exporters; call once, early in `main`.
pub fn init(config: &ObservabilityConfig, filter: EnvFilter, json: bool) -> ObservabilityGuard {
    // One line per finished operation, so spans are visible without a collector.
    let span_events = if config.span_events {
        FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    let console = if json {
        tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_span_events(span_events)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_span_events(span_events)
            .boxed()
    };

    let registry = tracing_subscriber::registry().with(filter).with(console);

    #[cfg(feature = "otel")]
    let otel_provider = {
        match config.otlp.as_ref().map(build_otel) {
            Some(Ok((layer, provider))) => {
                registry.with(layer).init();
                Some(provider)
            }
            Some(Err(error)) => {
                registry.init();
                tracing::error!(
                    target: "piramid::observability",
                    %error,
                    "OTLP exporter failed to start; continuing without it"
                );
                None
            }
            None => {
                registry.init();
                None
            }
        }
    };

    #[cfg(not(feature = "otel"))]
    {
        registry.init();
        if config.otlp.is_some() {
            tracing::warn!(
                target: "piramid::observability",
                "PIRAMID_OTLP_ENDPOINT is set but this build lacks the `otel` feature"
            );
        }
    }

    // Report what resolved, so a variable that did not take effect is visible at startup.
    tracing::info!(
        target: "piramid::observability",
        otlp = config.otlp.as_ref().map_or("off", |c| c.endpoint.as_str()),
        span_events = config.span_events,
        json_logs = json,
        "observability_ready"
    );

    ObservabilityGuard {
        #[cfg(feature = "otel")]
        otel: otel_provider,
    }
}

/// Builds the OTLP span-export layer and the provider that owns its background batcher.
#[cfg(feature = "otel")]
#[allow(clippy::type_complexity)]
fn build_otel<S>(
    cfg: &OtlpConfig,
) -> Result<
    (
        tracing_opentelemetry::OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>,
        opentelemetry_sdk::trace::SdkTracerProvider,
    ),
    Box<dyn std::error::Error>,
>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::Resource;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(cfg.endpoint.clone())
        .build()?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_attributes([KeyValue::new("service.name", cfg.service_name.clone())])
                .build(),
        )
        .build();

    let tracer = provider.tracer("piramid");
    opentelemetry::global::set_tracer_provider(provider.clone());
    Ok((tracing_opentelemetry::layer().with_tracer(tracer), provider))
}
