//! Cluster boundary for local-first and distributed routing.
//!
//! Local-only today. The abstraction exists so distributed placement and fan-out can be added
//! without leaking network concerns into services or storage.

mod node;
mod routing;

pub use node::{NodeCapabilities, NodeId, NodeRuntimeState};
pub use routing::{ClusterRouter, LocalClusterRouter, RouteDecision};
