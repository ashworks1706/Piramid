# piramid-model

Model execution and the embedding providers that feed it.

`inference` is the forward pass and its memory: weights, KV cache, batching, sampling. Scaffolding
today — every module is a boundary with nothing behind it yet.

`embeddings` turns text into vectors. Two providers: the OpenAI wire format, which covers OpenAI
itself and any server implementing it, and Ollama. LRU-cached and retried.

This crate depends on nothing in the retrieval stack. A strategy that actually queries an index is
a separate crate depending on both this one and `piramid-retrieval`, which is what keeps a
collection queryable with no model loaded.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
