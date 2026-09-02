//! Config status and reload responses.

use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigStatusResponse {
    pub app_config: piramid_core::config::Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reloaded_at: Option<u64>,
}

#[derive(Serialize)]
pub struct ConfigReloadResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reloaded_at: Option<u64>,
    pub app_config: piramid_core::config::Config,
}
