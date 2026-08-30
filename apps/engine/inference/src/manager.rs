//! The inference domain entry.

use crate::augment::{NoopRetrievalHook, RetrievalHook};

/// Owns the model runtime once one exists: weights, KV cache, batch queue, and the retrieval
/// hook the forward pass calls into. Scaffolded ahead of the code so drivers are written against
/// this surface rather than growing their own.
pub struct InferenceManager {
    hook: Box<dyn RetrievalHook>,
}

impl InferenceManager {
    /// A manager with retrieval fusion disabled.
    pub fn disabled() -> Self {
        Self {
            hook: Box::new(NoopRetrievalHook),
        }
    }

    /// A manager whose forward pass calls `hook` at the points it asks for.
    pub fn with_hook(hook: Box<dyn RetrievalHook>) -> Self {
        Self { hook }
    }

    /// The retrieval hook the forward pass consults.
    pub fn hook(&self) -> &dyn RetrievalHook {
        self.hook.as_ref()
    }
}

impl Default for InferenceManager {
    fn default() -> Self {
        Self::disabled()
    }
}
