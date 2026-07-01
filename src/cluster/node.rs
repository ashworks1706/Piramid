use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self("local".to_string())
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub cpu_threads: Option<usize>,
    pub memory_budget_bytes: Option<u64>,
    pub gpu_enabled: bool,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            cpu_threads: Some(num_cpus::get()),
            memory_budget_bytes: None,
            gpu_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeRuntimeState {
    pub id: NodeId,
    pub capabilities: NodeCapabilities,
    pub healthy: bool,
}

impl Default for NodeRuntimeState {
    fn default() -> Self {
        Self {
            id: NodeId::default(),
            capabilities: NodeCapabilities::default(),
            healthy: true,
        }
    }
}
