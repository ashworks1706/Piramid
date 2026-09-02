use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct LoggingConfig {
    #[serde(default = "crate::config::default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default = "crate::config::default_true")]
    pub config: bool,
    #[serde(default = "crate::config::default_true")]
    pub indexing: bool,
    #[serde(default = "crate::config::default_true")]
    pub search: bool,
    #[serde(default = "crate::config::default_true")]
    pub writes: bool,
    #[serde(default = "crate::config::default_true")]
    pub inference: bool,
    #[serde(default = "crate::config::default_true")]
    pub http: bool,
    /// Emit structured JSON lines instead of human-readable console output.
    #[serde(default)]
    pub json: bool,
    #[serde(default)]
    pub slow_query_ms: Option<u64>,
}

impl LoggingConfig {
    /// Threshold above which a query is logged at `warn`.
    pub fn slow_query_ms(&self) -> u64 {
        self.slow_query_ms.unwrap_or(DEFAULT_SLOW_QUERY_MS)
    }
}

/// Used both as the serde default and as the fallback for an explicit `null`.
const DEFAULT_SLOW_QUERY_MS: u64 = 500;

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: LogLevel::Info,
            config: true,
            indexing: true,
            search: true,
            writes: true,
            inference: true,
            http: true,
            json: false,
            slow_query_ms: Some(DEFAULT_SLOW_QUERY_MS),
        }
    }
}
