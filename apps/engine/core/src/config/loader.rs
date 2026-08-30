//! Configuration loading: file, then environment overrides.

use std::env;
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::config::{AppConfig, EmbeddingConfig};

/// Configuration could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// `CONFIG_FILE` could not be read or parsed.
    #[error("invalid configuration file: {0}")]
    File(String),
    /// An environment variable held a value the parser rejected.
    #[error("invalid environment configuration: {name}: {reason}")]
    Env {
        /// Variable name.
        name: String,
        /// What was wrong with it.
        reason: String,
    },
    /// The merged configuration failed validation.
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

/// Resolved runtime configuration: application settings plus process-level knobs.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub app: AppConfig,
    /// TCP port to bind.
    pub port: u16,
    pub data_dir: String,
    /// Queries slower than this are logged at `warn`.
    pub slow_query_ms: u128,
    pub embedding: Option<EmbeddingConfig>,
    /// Refuse writes below this much free disk, when set.
    pub disk_min_free_bytes: Option<u64>,
    /// Whether to drop to read-only rather than erroring when disk runs low.
    pub disk_readonly_on_low_space: bool,
}

/// Load configuration from `CONFIG_FILE`, then apply environment overrides.
pub fn load_app_config() -> Result<AppConfig, ConfigError> {
    let mut cfg = load_from_file()?.unwrap_or_default();

    cfg.apply_env_overrides()
        .map_err(|reason| ConfigError::Env {
            name: "<overrides>".to_string(),
            reason,
        })?;

    cfg.validate().map_err(ConfigError::Invalid)?;
    Ok(cfg)
}

/// Load everything the server needs: [`AppConfig`] plus env-driven runtime knobs.
pub fn load_runtime_config() -> Result<RuntimeConfig, ConfigError> {
    let app = load_app_config()?;

    let port = parse_env_or_default("PORT", 6333u16)?;
    let data_dir = env::var("DATA_DIR").or_else(|_| default_data_dir())?;
    let slow_query_default = u128::from(app.logging.slow_query_ms.unwrap_or(500));
    let slow_query_ms = parse_env_or_default("SLOW_QUERY_MS", slow_query_default)?;

    let embedding_provider = env::var("EMBEDDING_PROVIDER").ok();
    let embedding_model = env::var("EMBEDDING_MODEL").ok();
    let embedding_base_url = env::var("EMBEDDING_BASE_URL").ok();
    let embedding_api_key = env::var("OPENAI_API_KEY").ok();
    let embedding_timeout = parse_optional_env("EMBEDDING_TIMEOUT_SECS")?;

    let disk_min_free_bytes = parse_optional_env("DISK_MIN_FREE_BYTES")?;
    let disk_readonly_on_low_space = match env::var("DISK_READONLY_ON_LOW_SPACE") {
        Ok(value) => parse_bool_env("DISK_READONLY_ON_LOW_SPACE", &value)?,
        Err(_) => true,
    };

    let embedding = match embedding_provider {
        Some(provider) => {
            let Some(model) = embedding_model.or_else(|| default_embedding_model(&provider)) else {
                return Err(ConfigError::Env {
                    name: "EMBEDDING_MODEL".to_string(),
                    reason: format!("no default model for provider '{provider}'; set it"),
                });
            };
            Some(EmbeddingConfig {
                provider,
                model,
                api_key: embedding_api_key,
                base_url: embedding_base_url,
                options: serde_json::json!({}),
                timeout: embedding_timeout,
            })
        }
        None => None,
    };

    Ok(RuntimeConfig {
        app,
        port,
        data_dir,
        slow_query_ms,
        embedding,
        disk_min_free_bytes,
        disk_readonly_on_low_space,
    })
}

/// Default model for a provider that did not name one; unknown providers get none.
fn default_embedding_model(provider: &str) -> Option<String> {
    match provider {
        "openai" => Some("text-embedding-3-small".to_string()),
        "ollama" => Some("nomic-embed-text".to_string()),
        _ => None,
    }
}

fn load_from_file() -> Result<Option<AppConfig>, ConfigError> {
    let Ok(path) = env::var("CONFIG_FILE") else {
        return Ok(None);
    };
    let data = fs::read_to_string(&path)
        .map_err(|e| ConfigError::File(format!("failed to read CONFIG_FILE '{path}': {e}")))?;

    if path.ends_with(".yaml") || path.ends_with(".yml") {
        serde_yaml::from_str::<AppConfig>(&data)
            .map(Some)
            .map_err(|e| {
                ConfigError::File(format!("failed to parse YAML CONFIG_FILE '{path}': {e}"))
            })
    } else if path.ends_with(".json") {
        serde_json::from_str::<AppConfig>(&data)
            .map(Some)
            .map_err(|e| {
                ConfigError::File(format!("failed to parse JSON CONFIG_FILE '{path}': {e}"))
            })
    } else {
        Err(ConfigError::File(format!(
            "unsupported CONFIG_FILE extension for '{path}', expected .yaml, .yml, or .json"
        )))
    }
}

/// Default data directory: `~/.piramid`. Errors rather than falling back to the working directory.
pub fn default_data_dir() -> Result<String, ConfigError> {
    let home = env::var("HOME").map_err(|_| ConfigError::Env {
        name: "DATA_DIR".to_string(),
        reason: "not set, and HOME is unset so ~/.piramid cannot be resolved".to_string(),
    })?;
    let mut path = PathBuf::from(home);
    path.push(".piramid");
    Ok(path.to_string_lossy().to_string())
}

fn parse_env_or_default<T>(name: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => value.parse::<T>().map_err(|_| ConfigError::Env {
            name: name.to_string(),
            reason: format!("could not parse '{value}'"),
        }),
        Err(_) => Ok(default),
    }
}

fn parse_optional_env<T>(name: &str) -> Result<Option<T>, ConfigError>
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => value.parse::<T>().map(Some).map_err(|_| ConfigError::Env {
            name: name.to_string(),
            reason: format!("could not parse '{value}'"),
        }),
        Err(_) => Ok(None),
    }
}

fn parse_bool_env(name: &str, value: &str) -> Result<bool, ConfigError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError::Env {
            name: name.to_string(),
            reason: format!("expected 'true' or 'false', got '{value}'"),
        }),
    }
}
