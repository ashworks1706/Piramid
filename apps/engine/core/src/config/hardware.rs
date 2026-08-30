//! What the machine has, and how much of it to use.

use serde::{Deserialize, Serialize};

/// Which hardware to run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HardwareProfile {
    /// CPU, until GPU detection exists.
    #[default]
    Auto,
    /// Never touch the GPU.
    CpuOnly,
    /// Require a GPU; fail to start without one.
    Gpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct HardwareConfig {
    pub profile: HardwareProfile,

    /// Host memory the process will use. `None` is unbounded.
    pub memory_budget_bytes: Option<u64>,

    /// Device memory to claim. `None` is unbounded.
    pub gpu_memory_budget_bytes: Option<u64>,
}

impl HardwareConfig {
    /// Whether to acquire a device. The profile is the only switch; there is no separate flag to
    /// disagree with it.
    pub fn gpu_enabled(&self) -> bool {
        matches!(self.profile, HardwareProfile::Gpu)
    }
}
