//! Observability configuration, read from the environment.
//!
//! Every exporter is opt-in and disabled when its variable is unset, so a default deployment
//! sends nothing anywhere. That is the right default for a database people run on their own
//! hardware.

use std::env;

/// Where telemetry goes.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// OTLP trace exporter, when `PIRAMID_OTLP_ENDPOINT` is set.
    pub otlp: Option<OtlpConfig>,
    /// Sentry error reporting, when `PIRAMID_SENTRY_DSN` is set.
    pub sentry: Option<SentryConfig>,
}

/// OTLP trace export settings.
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// Collector endpoint, e.g. `http://localhost:4317`.
    pub endpoint: String,
    /// `service.name` attached to every span.
    pub service_name: String,
}

/// Sentry error-reporting settings.
#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// Project DSN.
    pub dsn: String,
    /// Environment tag, e.g. `production`.
    pub environment: String,
    /// Fraction of transactions sampled for performance monitoring, in `[0.0, 1.0]`.
    pub traces_sample_rate: f32,
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

        let sentry = env::var("PIRAMID_SENTRY_DSN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|dsn| SentryConfig {
                dsn,
                environment: env::var("PIRAMID_SENTRY_ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string()),
                traces_sample_rate: env::var("PIRAMID_SENTRY_TRACES_SAMPLE_RATE")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .filter(|rate: &f32| (0.0..=1.0).contains(rate))
                    .unwrap_or(0.1),
            });

        Self { otlp, sentry }
    }

    /// Whether any exporter is configured.
    pub fn is_enabled(&self) -> bool {
        self.otlp.is_some() || self.sentry.is_some()
    }
}
