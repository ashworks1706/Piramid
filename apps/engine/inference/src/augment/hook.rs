//! The retrieval seam: the trait a forward-pass driver calls into.

use piramid_core::error::Result;

/// Where in the forward pass a hook is being invoked.
///
/// Distinguishing these lets one hook implementation serve several fusion strategies without the
/// driver knowing which is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalPoint {
    /// Before the first decoder layer, once per sequence.
    SequenceStart,
    /// At a chunk boundary during decoding — the point at which a chunked-retrieval scheme queries
    /// the index for the chunk just completed.
    ChunkBoundary {
        /// Zero-based index of the chunk that just ended.
        chunk: usize,
    },
    /// Before a specific decoder layer, for schemes that fuse at a fixed depth.
    LayerEntry {
        /// Zero-based decoder layer index.
        layer: usize,
    },
}

/// Mutable view of the forward pass handed to a [`RetrievalHook`].
///
/// Deliberately minimal. It grows as the runtime grows; keeping it a named struct rather than a
/// long parameter list means adding state later does not change the trait signature and break
/// every implementation.
#[derive(Debug)]
pub struct ForwardContext<'a> {
    /// Where in the pass this invocation sits.
    pub point: RetrievalPoint,
    /// Tokens generated so far, for building a retrieval query.
    pub tokens: &'a [u32],
    /// Hidden state for the current position, laid out as `[batch, hidden_dim]`.
    ///
    /// A fusion implementation reads this to form its query and writes back the fused result.
    pub hidden_state: &'a mut [f32],
    /// Width of the hidden dimension.
    pub hidden_dim: usize,
}

/// A retrieval strategy that participates in the forward pass.
///
/// Implementations own their own index handles and encoders. The runtime knows only that it must
/// call [`RetrievalHook::on_retrieval_point`] at each point [`RetrievalHook::wants`] accepts.
pub trait RetrievalHook: Send + Sync {
    /// Name for logs and configuration.
    fn name(&self) -> &'static str;

    /// Whether this hook wants to run at `point`.
    ///
    /// Checked before assembling a [`ForwardContext`], so declining is cheap.
    fn wants(&self, point: RetrievalPoint) -> bool;

    /// Run retrieval and fuse the result into `ctx.hidden_state`.
    ///
    /// Called only when [`RetrievalHook::wants`] returned `true` for `ctx.point`.
    fn on_retrieval_point(&self, ctx: &mut ForwardContext<'_>) -> Result<()>;
}

/// A hook that never fires.
///
/// The default for a runtime with fusion disabled, and the control arm for any benchmark that
/// claims a fusion strategy helps.
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
