//! Telemetry export: tracing subscriber, optional OTLP spans, and Prometheus metrics.

pub mod config;
pub mod prometheus;

use std::sync::OnceLock;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub use crate::config::{LogLevel, LoggingConfig, OtlpConfig, TelemetryConfig};
pub use config::{ObservabilityError, ObservabilityResult};

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

/// Installs telemetry from configuration; call once, early in `main`.
///
/// Returns `None` when logging is disabled or a subscriber is already installed. The binary owns
/// *when* this happens; which directives a [`LoggingConfig`] implies is decided here, beside the
/// config that names them.
pub fn install(logging: LoggingConfig, telemetry: &TelemetryConfig) -> Option<ObservabilityGuard> {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    if INSTALLED.get().is_some() {
        return None;
    }
    INSTALLED.set(()).ok();
    if !logging.enabled {
        return None;
    }
    Some(init(telemetry, filter_for(logging), logging.json))
}

/// Turn a [`LoggingConfig`] into a filter. `RUST_LOG` replaces the level but not the per-target
/// switches, so turning one subsystem off stays possible alongside a custom level.
fn filter_for(logging: LoggingConfig) -> EnvFilter {
    let base = std::env::var("RUST_LOG").unwrap_or_else(|_| level_directive(logging.level).into());
    EnvFilter::new(directives(&base, logging))
}

/// Build the filter string: a base level, then one `=off` per subsystem switched off.
///
/// One string rather than directive-by-directive parsing, because every piece is a literal from
/// this file and there is no partial-failure case worth reporting.
fn directives(base: &str, logging: LoggingConfig) -> String {
    let mut out = vec![base.to_string()];
    for (enabled, target) in [
        (logging.config, "piramid::config"),
        (logging.indexing, "piramid::indexing"),
        (logging.search, "piramid::search"),
        (logging.writes, "piramid::writes"),
        (logging.inference, "piramid::inference"),
        (logging.http, "piramid::http"),
    ] {
        if !enabled {
            out.push(format!("{target}=off"));
        }
    }
    out.join(",")
}

/// Level name for the tracing filter syntax.
fn level_directive(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    }
}

/// Installs the tracing subscriber and any configured exporters.
fn init(config: &TelemetryConfig, filter: EnvFilter, json: bool) -> ObservabilityGuard {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_produce_a_bare_level() {
        assert_eq!(directives("info", LoggingConfig::default()), "info");
    }

    #[test]
    fn a_disabled_subsystem_becomes_an_off_directive() {
        let logging = LoggingConfig {
            search: false,
            http: false,
            ..LoggingConfig::default()
        };
        assert_eq!(
            directives("info", logging),
            "info,piramid::search=off,piramid::http=off"
        );
    }

    #[test]
    fn the_base_level_is_whatever_the_caller_resolved() {
        // RUST_LOG wins over the configured level, but never over the subsystem switches.
        let logging = LoggingConfig {
            indexing: false,
            ..LoggingConfig::default()
        };
        assert_eq!(
            directives("piramid=trace", logging),
            "piramid=trace,piramid::indexing=off"
        );
    }

    #[test]
    fn every_level_maps_to_a_tracing_name() {
        for (level, name) in [
            (LogLevel::Error, "error"),
            (LogLevel::Warn, "warn"),
            (LogLevel::Info, "info"),
            (LogLevel::Debug, "debug"),
            (LogLevel::Trace, "trace"),
        ] {
            assert_eq!(level_directive(level), name);
        }
    }
}
