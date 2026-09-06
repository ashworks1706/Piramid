//! Cluster boundary for local-first and distributed routing.
//!
//! Local-only today. Distributed placement and fan-out route through this boundary, keeping
//! network concerns out of services and storage.

mod node;
mod routing;

pub use node::{NodeCapabilities, NodeId, NodeRuntimeState};
pub use routing::{ClusterRouter, LocalClusterRouter, RouteDecision};
