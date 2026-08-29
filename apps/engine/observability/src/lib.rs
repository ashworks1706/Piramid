//! Telemetry export.
//!
//! `piramid_core::telemetry` *measures* — latency, lock contention, embedding throughput. This
//! crate *exports*: it builds the tracing subscriber and, when configured, ships spans over OTLP
//! and errors to Sentry. Keeping the two apart means adding an exporter never touches the code
//! that records a measurement.
//!
//! # Exporters
//!
//! Both are optional at compile time and disabled at runtime unless their environment variable is
//! set, so a default build sends nothing anywhere:
//!
//! | Feature | Variable | Effect |
//! |---|---|---|
//! | `otel` | `PIRAMID_OTLP_ENDPOINT` | Spans over OTLP |
//! | `sentry` | `PIRAMID_SENTRY_DSN` | Errors and panics to Sentry |
//!
//! `PIRAMID_LOG_SPANS=true` needs no feature: it logs one line per finished operation with its
//! duration and fields. Most operators will never run a collector, and this is what makes the
//! span instrumentation visible to them.
//!
//! OTLP is the wire format rather than any vendor's SDK, so Axiom, Grafana Tempo, Honeycomb, and
//! Jaeger all work from one configuration.
//!
//! Metrics are separate: [`prometheus`] renders what `piramid_core::telemetry` already aggregates,
//! served at `/metrics`. For a database, a scrape endpoint matters more than distributed tracing,
//! which is why it has no feature gate and no dependency.

pub mod config;
pub mod prometheus;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub use config::{ObservabilityConfig, OtlpConfig, SentryConfig};

/// Keeps exporters alive for the process lifetime.
///
/// Both exporters batch in the background, so dropping this flushes pending telemetry. Hold it in
/// `main` until shutdown — dropping it early silently stops export.
#[must_use = "dropping the guard shuts down telemetry export"]
pub struct ObservabilityGuard {
    #[cfg(feature = "sentry")]
    _sentry: Option<::sentry::ClientInitGuard>,
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
/// console logs. Call once, as early in `main` as possible — Sentry must be initialized before
/// the panic hook it installs can catch anything.
///
/// Exporter setup failures are logged and skipped rather than propagated: telemetry not reaching
/// a collector is not a reason to refuse to serve queries.
pub fn init(config: &ObservabilityConfig, filter: EnvFilter, json: bool) -> ObservabilityGuard {
    #[cfg(feature = "sentry")]
    let sentry_guard = config.sentry.as_ref().map(|cfg| {
        ::sentry::init((
            cfg.dsn.clone(),
            ::sentry::ClientOptions {
                environment: Some(cfg.environment.clone().into()),
                traces_sample_rate: cfg.traces_sample_rate,
                release: ::sentry::release_name!(),
                ..Default::default()
            },
        ))
    });

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

    #[cfg(feature = "sentry")]
    let registry = registry.with(sentry_tracing::layer());

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

    #[cfg(not(feature = "sentry"))]
    if config.sentry.is_some() {
        tracing::warn!(
            target: "piramid::observability",
            "PIRAMID_SENTRY_DSN is set but this build lacks the `sentry` feature"
        );
    }

    // Report what actually resolved. Telemetry that silently does nothing is worse than none,
    // and an operator who set a variable needs to see whether it took effect.
    tracing::info!(
        target: "piramid::observability",
        otlp = config.otlp.as_ref().map(|c| c.endpoint.as_str()).unwrap_or("off"),
        sentry = config.sentry.is_some(),
        span_events = config.span_events,
        json_logs = json,
        "observability_ready"
    );

    ObservabilityGuard {
        #[cfg(feature = "sentry")]
        _sentry: sentry_guard,
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
