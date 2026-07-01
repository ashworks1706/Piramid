//! Cluster boundary for local-first and distributed routing.
//!
//! Current runtime behavior is local-only. The routing abstraction exists so distributed
//! placement/fan-out can be added without leaking network concerns into services or storage.

mod node;
mod routing;

pub use node::{NodeCapabilities, NodeId, NodeRuntimeState};
pub use routing::{ClusterRouter, LocalClusterRouter, RouteDecision};
