//! The retrieval seam: the trait a forward-pass driver calls into.

use piramid_core::error::Result;

/// Where in the forward pass a hook is being invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalPoint {
    /// Before the first decoder layer, once per sequence.
    SequenceStart,
    /// Chunk boundary during decoding.
    ChunkBoundary {
        /// Zero-based index of the chunk that just ended.
        chunk: usize,
    },
    /// Before a specific decoder layer.
    LayerEntry {
        /// Zero-based decoder layer index.
        layer: usize,
    },
}

/// Mutable view of the forward pass handed to a [`RetrievalHook`].
#[derive(Debug)]
pub struct ForwardContext<'a> {
    /// Where in the pass this invocation sits.
    pub point: RetrievalPoint,
    /// Tokens generated so far, for building a retrieval query.
    pub tokens: &'a [u32],
    /// Hidden state for the current position, laid out as `[batch, hidden_dim]`.
    pub hidden_state: &'a mut [f32],
    /// Width of the hidden dimension.
    pub hidden_dim: usize,
}

/// A retrieval strategy that participates in the forward pass. An implementation must live in
/// its own crate — `inference` must never depend on the retrieval stack.
pub trait RetrievalHook: Send + Sync {
    /// Name for logs and configuration.
    fn name(&self) -> &'static str;

    /// Whether this hook wants to run at `point`.
    fn wants(&self, point: RetrievalPoint) -> bool;

    /// Run retrieval and fuse the result into `ctx.hidden_state`.
    fn on_retrieval_point(&self, ctx: &mut ForwardContext<'_>) -> Result<()>;
}

/// A hook that never fires; the default and the control arm for fusion benchmarks.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRetrievalHook;

impl RetrievalHook for NoopRetrievalHook {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn wants(&self, _point: RetrievalPoint) -> bool {
        false
    }

    fn on_retrieval_point(&self, _ctx: &mut ForwardContext<'_>) -> Result<()> {
        Ok(())
    }
}
