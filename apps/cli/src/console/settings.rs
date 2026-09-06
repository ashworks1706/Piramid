//! Console settings, read from the PIRAMID_CONSOLE__ prefix.
//!
//! The namespace is separate from the PIRAMID__ prefix the server reads. Every field has a
//! default.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the console looks and how often.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Base URL of the server, probed and opened with the open key.
    pub base_url: String,
    /// Base URL of the website, probed and opened with the open key.
    pub web_url: String,
    /// Lines kept in memory per unit.
    pub log_lines: usize,
    /// Directory for persistent unit logs, relative to the repo root unless absolute.
    pub log_dir: PathBuf,
    /// Time between probes.
    pub health_interval: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:6333".into(),
            web_url: "http://localhost:3000".into(),
            log_lines: 5000,
            log_dir: PathBuf::from("target/console-logs"),
            health_interval: Duration::from_secs(5),
        }
    }
}

impl Settings {
    /// Settings with every PIRAMID_CONSOLE__ override applied.
    ///
    /// An override that is set but unparseable returns an error rather than the default.
    pub fn from_env() -> Result<Self, SettingsError> {
        let mut settings = Self::default();
        if let Some(value) = var("BASE_URL") {
            settings.base_url = value;
        }
        if let Some(value) = var("WEB_URL") {
            settings.web_url = value;
        }
        if let Some(value) = var("LOG_LINES") {
            settings.log_lines = parse("LOG_LINES", &value)?;
        }
        if let Some(value) = var("LOG_DIR") {
            settings.log_dir = PathBuf::from(value);
        }
        if let Some(value) = var("HEALTH_INTERVAL_SECS") {
            settings.health_interval = Duration::from_secs(parse("HEALTH_INTERVAL_SECS", &value)?);
        }
        Ok(settings)
    }

    /// The log directory as an absolute path under root.
    pub fn log_dir_under(&self, root: &Path) -> PathBuf {
        if self.log_dir.is_absolute() {
            self.log_dir.clone()
        } else {
            root.join(&self.log_dir)
        }
    }
}

/// A setting that was given but could not be read.
#[derive(Debug, thiserror::Error)]
#[error("PIRAMID_CONSOLE__{key}: {value:?} is not a number")]
pub struct SettingsError {
    /// Name of the variable, without the prefix.
    key: &'static str,
    /// What it was set to.
    value: String,
}

fn var(key: &str) -> Option<String> {
    std::env::var(format!("PIRAMID_CONSOLE__{key}"))
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse<T: std::str::FromStr>(key: &'static str, value: &str) -> Result<T, SettingsError> {
    value.parse().map_err(|_| SettingsError {
        key,
        value: value.to_owned(),
    })
}

/// Walks up from start to the directory holding the justfile.
///
/// Returns None outside a checkout, where there is no justfile to find.
pub fn repo_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join("justfile").is_file() && dir.join("apps").is_dir())
        .map(Path::to_path_buf)
}
