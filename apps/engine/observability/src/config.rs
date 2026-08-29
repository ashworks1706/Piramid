//! Observability configuration, read from the environment.
//!
//! Every exporter is opt-in and disabled when its variable is unset, so a default deployment
//! sends nothing anywhere. That is the right default for a database people run on their own
//! hardware, and there is no endpoint here that this project controls.

use std::env;

/// Where telemetry goes.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// OTLP trace exporter, when `PIRAMID_OTLP_ENDPOINT` is set.
    pub otlp: Option<OtlpConfig>,
    /// Log a line when each instrumented operation finishes, with its duration and fields.
    ///
    /// Off by default: it roughly doubles log volume on a busy server. It exists because most
    /// operators will never run an OTLP collector, and without it the span instrumentation is
    /// invisible — a span only reaches the console when an event fires inside it, so a clean
    /// search produces no output at all.
    pub span_events: bool,
}

/// OTLP trace export settings.
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// Collector endpoint, e.g. `http://localhost:4317`.
    pub endpoint: String,
    /// `service.name` attached to every span.
    pub service_name: String,
}

impl ObservabilityConfig {
    /// Read configuration from the environment.
    ///
    /// Never fails: a malformed sample rate falls back to the default rather than preventing the
    /// server from starting. Telemetry misconfiguration should not take down the database.
    pub fn from_env() -> Self {
        let otlp = env::var("PIRAMID_OTLP_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|endpoint| OtlpConfig {
                endpoint,
                service_name: env::var("PIRAMID_OTLP_SERVICE_NAME")
                    .unwrap_or_else(|_| "piramid".to_string()),
            });

        let span_events = env::var("PIRAMID_LOG_SPANS")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

        Self { otlp, span_events }
    }

    /// Whether any exporter is configured.
    pub fn is_enabled(&self) -> bool {
        self.otlp.is_some()
    }
}
