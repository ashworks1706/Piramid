# piramid-embeddings

Embedding providers: the OpenAI wire format (including local servers that speak it) and Ollama.
Both are HTTP clients — this crate never loads a model.

A third, in-process provider is planned once `candle` lands (see `docs/ROADMAP.md`). It cannot
live here: it needs a model runtime, and this crate must not depend on `inference`. The binary
builds it and passes it to `EmbeddingsManager::with_embedder`.

Provider selection, caching, and retries behind one `Embedder` trait.

Separate from `piramid-inference` because embedding is a concrete ingestion feature today, while
local text generation is still a boundary with nothing behind it.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
