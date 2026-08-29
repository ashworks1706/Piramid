//! Telemetry export.
//!
//! `piramid_core::stats` holds what the engine measures about itself — latency, lock contention,
//! embedding throughput — as plain atomics with no dependency on `tracing` or any exporter. This
//! crate is where those measurements *go*: it builds the tracing subscriber and, when configured,
//! ships spans over OTLP.
//!
//! The split is load-bearing, not cosmetic. Recording is cheap and happens in `collections`,
//! `server`, and anywhere else; exporting pulls in `tracing-subscriber` and OpenTelemetry.
//! Merging them would link all of that into every crate that times a lock.
//!
//! # Exporters
//!
//! Both are optional at compile time and disabled at runtime unless their environment variable is
//! set, so a default build sends nothing anywhere:
//!
//! | Feature | Variable | Effect |
//! |---|---|---|
//! | `otel` | `PIRAMID_OTLP_ENDPOINT` | Spans over OTLP |
//!
//! `PIRAMID_LOG_SPANS=true` needs no feature: it logs one line per finished operation with its
//! duration and fields. Most operators will never run a collector, and this is what makes the
//! span instrumentation visible to them.
//!
//! OTLP is the wire format rather than any vendor's SDK, so Axiom, Grafana Tempo, Honeycomb, and
//! Jaeger all work from one configuration. That is the line this crate holds: it speaks open
//! standards — OTLP and the Prometheus exposition format — and integrates with no vendor's
//! product. Errors reach an operator as panics and `tracing` events on stderr, which their log
//! pipeline already collects.
//!
//! Metrics are separate: [`prometheus`] renders what `piramid_core::stats` already aggregates,
//! served at `/metrics`. For a database, a scrape endpoint matters more than distributed tracing,
//! which is why it has no feature gate and no dependency.

pub mod config;
pub mod prometheus;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub use config::{ObservabilityConfig, OtlpConfig};

/// Keeps exporters alive for the process lifetime.
///
/// Both exporters batch in the background, so dropping this flushes pending telemetry. Hold it in
/// `main` until shutdown — dropping it early silently stops export.
#[must_use = "dropping the guard shuts down telemetry export"]
pub struct ObservabilityGuard {
    // Held rather than dropped immediately: shutting the provider down is what flushes the last
    // batch of spans. `opentelemetry::global::shutdown_tracer_provider` was removed in 0.30, so
    // the provider itself is the handle.
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

/// Install the tracing subscriber and any configured exporters.
///
/// `filter` is the already-resolved `EnvFilter` for console output; `json` selects structured
/// console logs. Call once, as early in `main` as possible, so startup diagnostics are captured.
///
/// Exporter setup failures are logged and skipped rather than propagated: telemetry not reaching
/// a collector is not a reason to refuse to serve queries.
pub fn init(config: &ObservabilityConfig, filter: EnvFilter, json: bool) -> ObservabilityGuard {
    // FmtSpan::CLOSE emits one line per finished operation carrying its duration and recorded
    // fields. That is how the span instrumentation becomes visible without a collector.
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

    // Report what actually resolved. Telemetry that silently does nothing is worse than none,
    // and an operator who set a variable needs to see whether it took effect.
    tracing::info!(
        target: "piramid::observability",
        otlp = config.otlp.as_ref().map(|c| c.endpoint.as_str()).unwrap_or("off"),
        span_events = config.span_events,
        json_logs = json,
        "observability_ready"
    );

    ObservabilityGuard {
        #[cfg(feature = "otel")]
        otel: otel_provider,
    }
}

/// Build the OTLP span-export layer and the provider that owns its background batcher.
///
/// The provider is returned rather than forgotten because dropping it is what flushes.
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
