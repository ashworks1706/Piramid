# 0015 — The retrieval seam is device-aware and split into launch and join

**Context.** [ADR 0006](0006-retrieval-fusion-seam.md) defined `RetrievalHook` before anything
could call it, on the reasoning that a forward-pass driver written without the seam is hard to
retrofit with one. That reasoning only pays off if the seam is the right shape, and re-reading it
against the project's actual thesis — retrieval and inference sharing one device — it was not.

Two problems, both in the signature:

```rust
pub struct ForwardContext<'a> {
    pub point: RetrievalPoint,          // includes LayerEntry { layer }
    pub hidden_state: &'a mut [f32],
    ...
}
fn on_retrieval_point(&self, ctx: &mut ForwardContext<'_>) -> Result<()>;
```

`hidden_state` is a host slice. When the model's weights are on the device, the hidden state is a
device buffer, so the only way to satisfy this signature is to copy device→host, fuse, and copy
back — per invocation, and `LayerEntry` means per layer. The seam would have mandated exactly the
data movement co-location exists to remove.

And the call is synchronous: it must return before the pass continues. Overlapping search with
model compute on a separate stream — plausibly the largest win available in the co-located
design — could not be expressed at all, however the implementation was written.

**Decision.** The seam becomes device-aware and two-phase.

Hidden state is whichever it actually is:

```rust
pub enum HiddenState<'a> {
    Host(&'a mut [f32]),
    Device(&'a mut DeviceBuffer<f32>),
}
```

`inference` already depends on `gpu`, so this costs no new edge, and `DeviceBuffer` is ours rather
than a vendor type, so nothing leaks out of `gpu/backends/`. A hook that implements one path errors
on the other rather than silently copying — a transparent host fallback would hide the cost the
seam exists to avoid.

The single method splits:

```rust
fn launch(&self, request: &RetrievalRequest<'_>) -> Result<Box<dyn PendingRetrieval>>;
// ... driver does model work here ...
fn join(self: Box<Self>, ctx: &mut ForwardContext<'_>) -> Result<()>;
```

`RetrievalRequest` carries the tokens, the point and the model's stream, and deliberately not the
hidden state: launching cannot mutate the pass, so it cannot accidentally block on it. `join` gets
the mutable view, at the point the result is actually needed. On a device it should order the
model's stream against the hook's own rather than synchronizing the host, so waiting costs a stream
dependency instead of a stall.

**Why now.** The trait has no implementations, so this is a free change today and an expensive one
the moment a driver exists. `NoopRetrievalHook` gains a `NoopPending` half and stays the control arm
for fusion benchmarks.

**Consequences.** `inference` gains its first tests — three, covering launch-then-join fusion, that
`wants` gates the points a hook runs at, and that the noop hook leaves the pass untouched. The
crate previously had none, which is a poor state for the one module in it that is not a stub.

The device arm is unexercised until `cudarc` is wired, so it is a contract rather than a tested
path. That is the same bet ADR 0006 made and is the reason to get the shape right while it is free.

**Not done.** No `RetrievalPolicy` trait deciding *whether* to retrieve. `wants(point)` already
answers it for the cases in reach, and a third trait with one implementation and no measurements
behind it would be structure ahead of evidence. If the v0.4 benchmark shows retrieval frequency is
worth varying independently of the hook, it can be split then.
