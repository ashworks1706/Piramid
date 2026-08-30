//! The configuration file: two blocks, split by when a setting takes effect.

use serde::{Deserialize, Serialize};

use super::{CollectionConfig, RuntimeConfig, StartupConfig};

/// The whole of `config.yaml`.
///
/// The split is by lifecycle, not by subsystem: [`StartupConfig`] is baked into the process at
/// boot, [`RuntimeConfig`] is re-read on reload. Which block a setting is in is the answer to
/// "do I need to restart?", so it cannot drift the way a comment would.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub startup: StartupConfig,
    pub runtime: RuntimeConfig,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        self.startup.validate()?;
        self.runtime.validate()?;
        if self.startup.hardware.gpu_enabled()
            && matches!(self.runtime.execution, super::ExecutionMode::Scalar)
        {
            return Err(
                "startup.hardware.gpu_enabled conflicts with runtime.execution 'scalar'".into(),
            );
        }
        Ok(())
    }

    /// The defaults a newly created collection inherits.
    pub fn to_collection_config(&self) -> CollectionConfig {
        CollectionConfig {
            index: self.runtime.index.clone(),
            search: self.runtime.search,
            quantization: self.runtime.quantization,
            memory: self.runtime.memory,
            wal: self.runtime.wal,
            execution: self.runtime.execution,
            hardware: self.startup.hardware,
            limits: self.runtime.limits,
            cache: self.runtime.cache,
        }
    }
}
