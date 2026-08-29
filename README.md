<img width="1114" height="191" alt="Piramid Logo" src="https://github.com/user-attachments/assets/efaa4c47-62d1-4397-9899-8bd58d400fc6" />

<p align="center">
    <b>Inference Engine for Retrieval Systems</b>
</p>

<p align="center">
    <a href="https://crates.io/crates/piramid"><img src="https://img.shields.io/crates/v/piramid.svg" alt="crates.io"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="Apache 2.0"></a>
</p>

<p align="center">
  <a href="#what-this-is">What this is</a> •
  <a href="#quickstart">Quickstart</a> •
  <a href="#usage">Usage</a> •
  <a href="#where-this-is-going">Where this is going</a> •
  <a href="docs/ARCHITECTURE.md">Architecture</a> •
  <a href="docs/SETUP.md">Setup</a> •
  <a href="docs/ROADMAP.md">Roadmap</a>
</p>

## What this is

Today Piramid is a single-node vector database written in Rust. Collections, kNN and range
search, metadata filtering, embedding ingestion, WAL durability, three ANN index families, and
SIMD distance kernels. It runs as one binary with no external dependencies.

The longer-term goal is to run retrieval and transformer inference in the same process, with
retrieved vectors entering the model through cross-attention during the forward pass rather than
being pasted into the prompt. That half is scaffolding: the seams are defined, there is no
implementation behind them yet. See [Where this is going](#where-this-is-going).

https://github.com/user-attachments/assets/487cbc0f-c279-4a15-a160-9acd4666fbe6

### How it's put together

Eleven library crates under `apps/engine`, plus the binary that links them. A crate may depend on
one below it in the list; the reverse fails CI.

```mermaid
flowchart TD
    CLI[apps/cli]
    Server[server<br/>http · services · state]
    Inference[inference<br/>forward pass · retrieval seam]
    Collections[collections]
    Embeddings[embeddings]
    Search[search]
    Index[index<br/>flat · hnsw · ivf]
    Storage[storage<br/>records · WAL · mmap]
    Core[core<br/>error · config · metadata]
    Compute[compute<br/>distance kernels]
    Gpu[gpu<br/>device · buffer · stream]

    CLI --> Server
    CLI --> Inference
    Server --> Collections
    Server --> Embeddings
    Inference --> Gpu
    Collections --> Search
    Search --> Index
    Index --> Storage
    Index --> Compute
    Storage --> Core
    Core --> Compute
```

`compute` and `gpu` depend on nothing else in the workspace. `gpu` owns the device runtime so that
both `compute` and `inference` can share one device, which is what lets vectors and model weights
live in the same address space later.

Full guide in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Decisions and their reasoning in
[docs/decisions/](docs/decisions/).

## Quickstart

```bash
cargo install piramid
piramid serve --data-dir ./data
```

Listens on `0.0.0.0:6333`. Data goes to `~/.piramid` unless `DATA_DIR` says otherwise. Every
setting is listed in [`.env.example`](.env.example).

With Docker:

```bash
just up
```

From source:

```bash
just bootstrap   # .env, git hooks, dependencies
just check       # fmt, clippy, tests, layering
just serve
```

## Usage

```bash
# Create a collection
curl -X POST http://localhost:6333/api/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "docs"}'

# Store a vector
curl -X POST http://localhost:6333/api/collections/docs/vectors \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, 0.3, 0.4], "text": "Hello world", "metadata": {"category": "greeting"}}'

# Embed and store text (needs EMBEDDING_PROVIDER set)
curl -X POST http://localhost:6333/api/collections/docs/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["hello", "bonjour"], "metadata_list": [{"lang": "en"}, {"lang": "fr"}]}'

# Search
curl -X POST http://localhost:6333/api/collections/docs/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, 0.3, 0.4], "k": 5}'
```

Operational endpoints: `/api/health`, `/api/readyz`, `/api/version`, `/api/metrics` for the JSON
view and `/metrics` for Prometheus.

## Where this is going

The idea is that knowledge does not have to live in a model's weights, and it does not have to
live in the prompt either. Retrieval that reaches the model through cross-attention costs no
context window, and it can happen during generation rather than once before it.

Piramid commits to the seam for that rather than to a particular mechanism.
`inference::augment::RetrievalHook` says when retrieval may happen and what it may touch, not how
retrieved data gets combined. Chunked cross-attention, residual-stream gating, and learned index
routing would all be implementations of the same trait. The trait exists before anything calls it
because a forward-pass driver written without the seam is hard to retrofit with one.

The evidence for the specific RETRO mechanism is mixed, and
[ADR 0006](docs/decisions/0006-retrieval-fusion-seam.md) lays out the case against it alongside
the case for, plus the experiment that would settle it. Building the seam rather than the
mechanism means the device runtime, vector layout, kernel dispatch, and indexes stay useful
whichever way that goes.

[docs/ROADMAP.md](docs/ROADMAP.md) has the plan, including a list of what is currently missing or
broken.

## Contributing

`just doctor` checks your setup and `just check` is the gate. [AGENTS.md](AGENTS.md) covers the
layout, the dependency rule, and the conventions.

## License

[Apache 2.0](LICENSE)
