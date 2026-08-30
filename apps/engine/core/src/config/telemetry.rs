//! Tracing and metrics export settings.

use serde::{Deserialize, Serialize};

/// Where traces and metrics go. Installed once, when the process starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct TelemetryConfig {
    /// OTLP trace export. `None` disables it.
    pub otlp: Option<OtlpConfig>,

    /// Log a line when each instrumented operation finishes, with its duration and fields.
    pub span_events: bool,
}

/// OTLP trace export settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtlpConfig {
    /// Collector endpoint, e.g. `http://localhost:4317`.
    pub endpoint: String,

    /// `service.name` attached to every span.
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

fn default_service_name() -> String {
    "piramid".to_string()
}
