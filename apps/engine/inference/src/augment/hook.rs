//! The retrieval seam: the trait a forward-pass driver calls into.

use piramid_core::error::Result;
use piramid_gpu::{DeviceBuffer, Stream};

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

/// Where the hidden state for the current position lives.
///
/// The whole point of running retrieval and inference in one process is that neither has to copy
/// through host memory to reach the other. A seam that could only hand out `&mut [f32]` would force
/// a device-to-host-to-device round trip on every invocation — per layer, at `LayerEntry` — which
/// costs more than the fusion saves. So the state is whichever it actually is, and a hook that only
/// implements one path says so by erroring on the other.
#[derive(Debug)]
pub enum HiddenState<'a> {
    /// Host memory, laid out as `[batch, hidden_dim]`. The CPU path, and the only one a default
    /// build can produce.
    Host(&'a mut [f32]),
    /// Device memory, same layout. Fusing into this must stay on the device.
    Device(&'a mut DeviceBuffer<f32>),
}

/// Read-only view handed to [`RetrievalHook::launch`]: enough to build a query, and deliberately
/// no access to the hidden state, so launching cannot mutate the pass or block waiting for it.
#[derive(Debug)]
pub struct RetrievalRequest<'a> {
    /// Where in the pass this invocation sits.
    pub point: RetrievalPoint,
    /// Tokens generated so far, for building a retrieval query.
    pub tokens: &'a [u32],
    /// Width of the hidden dimension.
    pub hidden_dim: usize,
    /// The stream the model's own work is queued on, when running on a device. A hook that issues
    /// device work should use a stream of its own so the two overlap, and order against this one
    /// only in [`PendingRetrieval::join`].
    pub stream: Option<&'a Stream>,
}

/// Mutable view of the forward pass, handed to [`PendingRetrieval::join`] at the point the result
/// is actually needed.
#[derive(Debug)]
pub struct ForwardContext<'a> {
    /// Where in the pass this invocation sits.
    pub point: RetrievalPoint,
    /// Hidden state for the current position.
    pub hidden: HiddenState<'a>,
    /// Width of the hidden dimension.
    pub hidden_dim: usize,
    /// The model's stream, when running on a device.
    pub stream: Option<&'a Stream>,
}

/// Retrieval that has been started and not yet fused.
///
/// Held by the driver across whatever model work it can do in the meantime. Splitting the seam here
/// is what makes overlap expressible at all: a single `retrieve_and_fuse` call would serialize
/// search behind the forward pass no matter how the implementation was written.
pub trait PendingRetrieval: Send {
    /// Wait for the result and fuse it into `ctx.hidden`.
    ///
    /// On a device this should order the model's stream against the hook's own rather than
    /// synchronizing the host, so the wait costs a stream dependency instead of a stall.
    fn join(self: Box<Self>, ctx: &mut ForwardContext<'_>) -> Result<()>;
}

/// A retrieval strategy that participates in the forward pass. An implementation must live in
/// its own crate — `inference` must never depend on the retrieval stack.
pub trait RetrievalHook: Send + Sync {
    /// Name for logs and configuration.
    fn name(&self) -> &'static str;

    /// Whether this hook wants to run at `point`.
    fn wants(&self, point: RetrievalPoint) -> bool;

    /// Start retrieval. Must return without waiting for the result.
    fn launch(&self, request: &RetrievalRequest<'_>) -> Result<Box<dyn PendingRetrieval>>;
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

    fn launch(&self, _request: &RetrievalRequest<'_>) -> Result<Box<dyn PendingRetrieval>> {
        Ok(Box::new(NoopPending))
    }
}

/// The pending half of [`NoopRetrievalHook`]; joining it leaves the pass untouched.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPending;

impl PendingRetrieval for NoopPending {
    fn join(self: Box<Self>, _ctx: &mut ForwardContext<'_>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "assertions in tests")]

    use super::*;

    /// Stands in for a real hook: records what it was asked for, then adds a constant on join.
    struct AddOne;

    struct AddOnePending {
        launched_at: RetrievalPoint,
    }

    impl RetrievalHook for AddOne {
        fn name(&self) -> &'static str {
            "add-one"
        }

        fn wants(&self, point: RetrievalPoint) -> bool {
            matches!(point, RetrievalPoint::SequenceStart)
        }

        fn launch(&self, request: &RetrievalRequest<'_>) -> Result<Box<dyn PendingRetrieval>> {
            Ok(Box::new(AddOnePending {
                launched_at: request.point,
            }))
        }
    }

    impl PendingRetrieval for AddOnePending {
        fn join(self: Box<Self>, ctx: &mut ForwardContext<'_>) -> Result<()> {
            assert_eq!(self.launched_at, ctx.point);
            match &mut ctx.hidden {
                HiddenState::Host(state) => {
                    for slot in state.iter_mut() {
                        *slot += 1.0;
                    }
                    Ok(())
                }
                HiddenState::Device(_) => Ok(()),
            }
        }
    }

    #[test]
    fn a_hook_launches_then_fuses_on_join() {
        let hook = AddOne;
        let tokens = [1u32, 2, 3];
        let mut hidden = vec![0.0f32; 4];

        let request = RetrievalRequest {
            point: RetrievalPoint::SequenceStart,
            tokens: &tokens,
            hidden_dim: hidden.len(),
            stream: None,
        };
        // The driver may do model work between these two calls; that is the point of the split.
        let pending = hook.launch(&request).unwrap();

        let mut ctx = ForwardContext {
            point: RetrievalPoint::SequenceStart,
            hidden_dim: hidden.len(),
            hidden: HiddenState::Host(&mut hidden),
            stream: None,
        };
        pending.join(&mut ctx).unwrap();

        assert_eq!(hidden, vec![1.0; 4]);
    }

    #[test]
    fn wants_gates_the_points_a_hook_runs_at() {
        let hook = AddOne;
        assert!(hook.wants(RetrievalPoint::SequenceStart));
        assert!(!hook.wants(RetrievalPoint::LayerEntry { layer: 0 }));
        assert!(!hook.wants(RetrievalPoint::ChunkBoundary { chunk: 0 }));
    }

    #[test]
    fn the_noop_hook_leaves_the_pass_untouched() {
        let hook = NoopRetrievalHook;
        let tokens = [7u32];
        let mut hidden = vec![0.5f32; 3];

        assert!(!hook.wants(RetrievalPoint::SequenceStart));
        let pending = hook
            .launch(&RetrievalRequest {
                point: RetrievalPoint::SequenceStart,
                tokens: &tokens,
                hidden_dim: hidden.len(),
                stream: None,
            })
            .unwrap();

        let mut ctx = ForwardContext {
            point: RetrievalPoint::SequenceStart,
            hidden_dim: hidden.len(),
            hidden: HiddenState::Host(&mut hidden),
            stream: None,
        };
        pending.join(&mut ctx).unwrap();

        assert_eq!(hidden, vec![0.5; 3]);
    }
}
