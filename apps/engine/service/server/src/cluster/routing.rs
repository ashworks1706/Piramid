use crate::cluster::{NodeId, NodeRuntimeState};

#[derive(Debug, Clone)]
pub enum RouteDecision {
    Local,
    Remote(NodeId),
}

pub trait ClusterRouter: Send + Sync {
    fn local_node(&self) -> NodeRuntimeState;
    fn route_collection(&self, collection: &str) -> RouteDecision;
}

#[derive(Debug, Clone, Default)]
pub struct LocalClusterRouter {
    local: NodeRuntimeState,
}

impl LocalClusterRouter {
    pub fn new(local: NodeRuntimeState) -> Self {
        Self { local }
    }
}

impl ClusterRouter for LocalClusterRouter {
    fn local_node(&self) -> NodeRuntimeState {
        self.local.clone()
    }

    fn route_collection(&self, _collection: &str) -> RouteDecision {
        RouteDecision::Local
    }
}
