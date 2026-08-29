# piramid-embeddings

Embedding providers: OpenAI, Ollama, local.

Provider selection, caching, and retries behind one `Embedder` trait.

Separate from `piramid-inference` because embedding is a concrete ingestion feature today, while
local text generation is still a boundary with nothing behind it.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md) for how the crates fit together.
