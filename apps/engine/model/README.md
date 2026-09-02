# piramid-model

Model execution, the seam retrieval enters it through, and the providers that feed it.

`inference` is the forward pass and its memory: weights, KV cache, batching, sampling.
Scaffolding today — every module is a boundary with nothing behind it yet.

`fusion` is the `RetrievalHook` seam. `HiddenState` is either a host slice or a
`DeviceBuffer`, so fusing into device memory needs no host round trip, and `launch` is separate
from `join` so retrieval can run on its own stream while the model computes.

`embeddings` turns text into vectors. Two providers: the OpenAI wire format, which covers OpenAI
itself and any server implementing it, and Ollama. LRU-cached and retried.

This crate depends on nothing in the retrieval stack, which is what keeps a collection queryable
with no model loaded. A hook implementation that queries an index is a separate crate depending on
both this one and `piramid-database`.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
