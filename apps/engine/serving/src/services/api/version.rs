//! Build identity.

use serde::Serialize;

/// Binary version, plus the build's git hash when one was baked in.
#[derive(Serialize)]
pub struct VersionResponse {
    pub version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<&'static str>,
}
