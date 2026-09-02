# piramid-fusion

The seam where retrieval enters the forward pass.

Defined apart from both halves on purpose. A model runtime that depends on the retrieval stack
cannot be built without it; a retrieval stack that depends on a model runtime stops being queryable
with no model loaded. An implementation depends on this crate, `retrieval` and `model` — none of
them depend on it.

`HiddenState` is either a host slice or a `DeviceBuffer`, so fusing into device memory needs no
host round trip. `launch` is separate from `join` so retrieval can run on its own stream while
the model computes.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
