//! Observability configuration, read from the environment.

use std::env;

/// Where telemetry goes.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// OTLP trace exporter, when `PIRAMID_OTLP_ENDPOINT` is set.
    pub otlp: Option<OtlpConfig>,
    /// Log a line when each instrumented operation finishes, with its duration and fields.
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
    pub fn from_env() -> Result<Self, String> {
        let otlp = env::var("PIRAMID_OTLP_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|endpoint| OtlpConfig {
                endpoint,
                service_name: env::var("PIRAMID_OTLP_SERVICE_NAME")
                    .unwrap_or_else(|_| "piramid".to_string()),
            });

        let span_events = match env::var("PIRAMID_LOG_SPANS") {
            Ok(value) => match value.as_str() {
                "true" => true,
                "false" => false,
                other => {
                    return Err(format!(
                        "PIRAMID_LOG_SPANS: expected 'true' or 'false', got '{other}'"
                    ))
                }
            },
            Err(_) => false,
        };

        Ok(Self { otlp, span_events })
    }
}
