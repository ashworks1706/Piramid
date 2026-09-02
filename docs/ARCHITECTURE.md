# Architecture

How the workspace is cut, why each boundary sits where it does, and what has to stay true.

## The problem this shape solves

Piramid runs retrieval and, eventually, transformer inference in one process, on one device, so
that a retrieval costs microseconds rather than milliseconds. That price is the product: a lookup
you can only afford once is a query, and one you can afford sixteen times per generation is
recall. Retrieval inside a generation — repeated, overlapped with compute, against device-resident
state — does not cross a service boundary; retrieval before prefill costs one hop and does not
need one.

Colocation is not the same thing. Putting a vector database and an inference server on one host
removes the network, but the candidate set still travels to reach the weights. What makes a
retrieval cheap enough to run continuously is the slab already sitting in device memory beside
them, which is only possible in one address space.

That single-process goal is why internal boundaries matter: with no network between the layers, nothing
keeps them from growing into each other except discipline, and discipline that nothing checks tends
not to survive.

So the layering is physical. Each layer is a crate, and `scripts/check-deps.sh` fails CI on an edge
that isn't in the rule below.

## The tree

```text
apps/                     everything we author
  engine/                 the library crates, one folder each
    core/                 errors, config, document, metadata, validation, stats, observability
    hardware/             compute, gpu, quantization
    database/             storage, index, search, and the Collection over them
    model/                inference, fusion, embeddings
    serving/              how the outside world reaches it
  cli/                    the piramid binary, which links the engine into one artifact
  website/                piramiddb.com, with blog content and images inside it
  sdk/                    npm and python clients

deploy/  docs/  scripts/  .claude/  .github/     how it's built, shipped, and explained
```

Two things the naming is doing. `engine/` says what the thing is; "crates" describes Rust's
compilation model, not the product. And one binary doesn't mean one folder: the engine is eleven
crates and `apps/cli` is what links them into an artifact.

Each cut is a real one.

`hardware` is the code that changes when the machine changes. `compute` owns what cosine means and
which strategy runs it, `gpu` owns the device — contexts, buffers, streams, modules — and
`quantization` owns the encodings both score over. It is a leaf, so kernels can be benchmarked on
their own and `model` can get a device without reaching through retrieval math.

`database` is where vectors live: records, WAL, mmap, sidecars. It decides nothing about API
behaviour or collection lifecycle — a `RecordStore` does not know what a collection is.

`retrieval` is how they are found: `index` for the ANN structure, `search` for planning and
scoring. `collections` is the object that owns a store, a cache, a checkpoint policy and an index;
a collection is acted on by search rather than being a way of finding things itself.

`model` is the forward pass, the `fusion` seam retrieval enters it through, and the `embeddings`
providers that turn text into a vector. It depends on nothing in the retrieval stack, which is what
keeps a collection queryable with no model loaded.

`core` is the vocabulary everything shares: errors, the whole configuration surface, document
metadata and its filters, validation, and the counters the engine keeps about itself.

`core::stats` and `core::observability` split a concern that is easy to read as two names for one
thing. `stats` is what the engine measures: latency, lock contention, embedding throughput, held as
plain atomics with no dependency on an exporter, so any crate can record into it freely.
`observability` is where those numbers go, and it carries `tracing-subscriber` and OpenTelemetry.

One folder per crate, no grouping folders. The tree carries names; the layering is the
dependency rule, enforced by `scripts/check-deps.sh`. Folder order is not dependency order — `core`
depends on `compute` for the `ExecutionMode` and `Metric` types that configuration carries.

## Crates

```mermaid
flowchart TD
    CLI[apps/cli<br/>binary + umbrella facade]
    Serving[serving<br/>http · services · api · state · cluster]
    Model[model<br/>inference · fusion · embeddings]
    Database[database<br/>storage · index · search · Collection]
    Core[core<br/>error · config · document · metadata · stats · observability]
    Hardware[hardware<br/>compute · gpu · quantization]

    CLI --> Serving
    Serving --> Database
    Serving --> Model
    Serving --> Core
    Database --> Core
    Database --> Hardware
    Model --> Core
    Model --> Hardware
    Core --> Hardware
```

| Crate | Owns | Must not |
|---|---|---|
| `core` | Every error the app wraps, all configuration, the document and hit shapes, metadata and its filters, validation, `stats`, and the telemetry export those feed | Know about HTTP or end the process |
| `hardware` | Distance math and strategy dispatch, the device runtime, quantization encodings | Depend on anything in the workspace, or let vendor types escape `gpu::backends` |
| `database` | Records, WAL, sidecars, mmap; ANN traversal and the sidecar format; query planning, filtering, scoring, ranking; the `Collection`, its caches, checkpoint and compaction | Serve HTTP |
| `model` | Model execution, KV cache, batching, sampling; the `RetrievalHook` seam; embedding providers | Depend on `database`, or be required for retrieval to work |
| `serving` | Routes, handlers, services, wire shapes, `AppState`, routing | Touch file formats or index internals |
| `apps/cli` | Argument parsing, process lifecycle, terminal output | Contain domain logic |

## What it's built on

Rust 1.87, edition 2021. One binary, no runtime services to install alongside it: the storage
engine, the indexes and the HTTP server are all in-process.

| Area | Crate | Why this one |
|---|---|---|
| HTTP | `axum`, `tower-http` | Rides on `hyper` and the `tower` middleware ecosystem, so timeouts, CORS and tracing are layers rather than framework features |
| Async | `tokio` | What `axum` and `reqwest` already require; a second runtime would mean two thread pools |
| Serialization | `serde` with `serde_json`, `serde_yaml`, `bincode` | JSON on the wire and in the WAL, YAML for config, `bincode` for the index sidecars where size matters more than readability |
| Errors | `thiserror` | Typed enums per layer. No `anyhow` in libraries: a caller has to be able to match on the failure |
| SIMD | `wide` | `f32x8` that lowers to AVX2 on x86_64 and NEON on aarch64, so one kernel covers both without intrinsics |
| Parallelism | `rayon` | Work-stealing for the batch kernels, kept out of the hot single-pair path |
| Storage | `memmap2` | Reads records without copying them into the heap first |
| Concurrency | `dashmap`, `parking_lot`, `lru` | Sharded map so unrelated collections don't contend, smaller and faster locks, bounded caches |
| Telemetry | `tracing`, OpenTelemetry, OTLP | Open standards only, no vendor SDK |
| CLI | `clap` | Derive API, so the parser and the help text cannot drift apart |
| Benchmarks | `criterion` | Statistical comparison, which matters when a kernel change is worth single-digit percent |

Two features are reserved for hardware and model runtimes. Both are additive, off by default, and
currently empty — the backend modules behind them are stubs, so `cargo build` needs no CUDA
toolkit and no model runtime, and `CudaStrategy::is_available` reports `false` rather than
pretending:

| Feature | Intended crate | For |
|---|---|---|
| `gpu-cuda` | `cudarc` | Device runtime in `gpu`, confined to `gpu/backends/` |
| `inference-candle` | `candle` | Model execution in `inference`, confined to `inference/backends/` |

When those land, the vendor types stay inside those backend modules. Nothing above them imports
either crate, which is what allows a second backend later without touching the layers between.

The website is separate and ships nothing: Next.js 16 on React 19, TypeScript, Tailwind 4, and
MDX through `next-mdx-remote` with KaTeX for the maths in the blog posts.

## The dependency rule

A crate may depend on one listed below it. The reverse is a violation.

```
hardware ─→ core ─┬─→ database ─→ serving ─→ cli
                  └─→ model ────┘
```

`scripts/check-deps.sh` holds the allow-list. Adding an edge means editing that file and this
document in the same change.

### Why compute and gpu are leaves

Neither depends on anything in the workspace, `core` included.

`compute` is a leaf because kernels should be liftable into a standalone benchmark, and because a
kernel layer that imports application configuration can't be reasoned about on its own.
`ExecutionMode` lives in `compute` and is re-exported by `config` rather than the other way round;
it names a backend, which is a compute concern.

`gpu` is a leaf because both `compute` and `inference` need a device. If the device runtime lived
inside `compute`, inference would depend on retrieval math to allocate memory. Keeping `gpu` a peer
means both share one `Device`, which is what puts vectors and model weights in the same address
space.

```
compute/strategies/cuda.rs ─┐
                            ├──→ gpu  (Device, DeviceBuffer, Stream, KernelModule)
inference/backends/*.rs   ──┘
```

## The three seams

Everything else exists to make these cheap to implement and swap.

### compute::DistanceKernels

One strategy per file in `compute/strategies/`, one arm in the registry. Nothing else changes.
"Backends" means the vendor layer — `gpu/backends/`, `inference/backends/` — and nothing else.

```rust
fn cosine_batch(&self, query: &[f32], candidates: &[f32], dim: usize, out: &mut [f32])
    -> ComputeResult<()>;
```

`candidates` is a contiguous row-major slab, not `&[Vec<f32>]`. A slab uploads to a device in one
memcpy; a slice of `Vec`s is a set of scattered allocations that a device backend would have to
gather on every call, and that gather costs more than the kernel saves. `out` is caller-owned so
the buffer can be reused across queries and pinned later for async transfer.

CPU backends get correct batch behaviour from default implementations that loop over the pairwise
methods. A device backend overrides them with a real launch.

`backends::for_mode` is the only lookup and it checks availability itself. A mode naming a
backend this build does not contain, or this machine cannot run, is an error — there is no
fallback, because a caller that asked for one backend and silently got another has no way to know
its numbers came from somewhere else. Callers resolve once per operation and pass the backend into
the loop.

### storage::vectors::VectorReader

How an index reads vectors it doesn't own, so the backing store can change without touching any
index. Cache-backed today, slab-backed or mmap-backed later.

`as_slab() -> Option<(&[f32], usize)>` is the fast path. A reader over scattered allocations
returns `None` rather than silently copying, because hiding that cost would make the CPU/device
choice unmeasurable. `gather_into()` is the portable fallback. Both have defaults.

A contiguous store — one `Vec<f32>` at a fixed stride, with a `Uuid → u32` ordinal map so hot
structures hold a 4-byte handle instead of a 16-byte key — is what makes `as_slab` return `Some`.
It does not exist yet: making `cache::VectorStore` contiguous is a v0.3.0 roadmap item, and
because `as_slab` is optional it can land one reader at a time.

### model::fusion::RetrievalHook

Where retrieval enters the forward pass.

```rust
fn wants(&self, point: RetrievalPoint) -> bool;
fn launch(&self, request: &RetrievalRequest<'_>) -> Result<Box<dyn PendingRetrieval>>;
// ... the driver does model work here ...
fn join(self: Box<Self>, ctx: &mut ForwardContext<'_>) -> Result<()>;
```

Mechanism-agnostic on purpose: it says when retrieval may occur and what it may touch, not how
retrieved data gets combined. Chunked cross-attention, residual-stream gating, and learned index
routing would all be implementations of the same trait.

Two things in that signature are load-bearing. `ForwardContext` carries a `HiddenState` that is either a host slice or a `DeviceBuffer`, because a
host-only seam would force a device-to-host-to-device copy per invocation — per layer, at
`LayerEntry` — which is exactly the data movement co-locating retrieval and inference exists to
remove. And the split into `launch` and `join` is what lets search overlap model compute on its own
stream; a single fused call serializes them no matter how it is implemented.

It exists before anything calls it because a forward-pass driver written without the seam is hard
to retrofit with one, and a driver written with it costs nothing extra. `RetrievalRequest` withholds
the hidden state so launching cannot block on the pass it is meant to overlap.

A strategy that actually queries an index depends on `search`, so it belongs in its own crate
depending on both that and `inference`. That's what keeps `inference` free of the retrieval stack.

## Request flow

```mermaid
sequenceDiagram
    participant C as Client
    participant H as server::http
    participant S as server::services
    participant M as CollectionManager
    participant Col as Collection
    participant Se as search
    participant Idx as index
    participant St as storage

    C->>H: HTTP request
    H->>S: typed DTO
    S->>M: get_existing / get_or_create
    M->>Col: open or return loaded
    S->>Col: domain operation
    Col->>Se: SearchTarget + params
    Se->>Idx: IndexSearchRequest
    Idx->>St: read vectors via VectorReader
    Se-->>Col: ranked hits
    Col-->>S: domain result
    S-->>H: response DTO
    H-->>C: JSON
```

Conversion boundaries are explicit: HTTP shapes in `server::http`, operational decisions in
`server::services`, domain mutation in `collections`, bytes and files in `storage`.

`search` takes a `SearchTarget` — index, readers, defaults — rather than a `Collection`. That's
what keeps `search` below `collections` instead of circular with it.

## Write path

The record store plus sidecars are the source of truth. Cache and index are acceleration
structures that have to stay rebuildable from stored records.

```mermaid
flowchart TD
    A[service receives write] --> B[CollectionManager opens collection]
    B --> C[validate limits and dimensions]
    C --> D[CheckpointManager logs WAL entry]
    D --> E[RecordStore appends document]
    E --> F[update offset index]
    F --> G[CacheManager updates hot vector and metadata]
    G --> H[VectorIndex updates ANN structure]
    H --> I[checkpoint condition may flush sidecars]
```

## Durability

WAL plus checkpoints. `CheckpointManager` owns collection-level bookkeeping and WAL rotation;
byte-level serialization stays in `storage`. On open, the builder loads sidecars, opens the record
store, initializes the WAL, and replays if needed.

An index owns its own sidecar format, so save and load live in `index::persistence` rather than in
`storage`.

## Configuration

One file, two blocks, split by *when a setting takes effect* rather than by which subsystem owns
it:

```yaml
startup:   # applied once at boot; changing one needs a restart
runtime:   # re-read on POST /config/reload
```

The split is by lifecycle because the alternative had already produced a bug. Grouping by
subsystem put `logging` inside the reloadable object while OTLP settings lived in a separate
env-only struct in `observability` — and neither is re-read after boot, so a reload returned 200
and silently changed nothing. Which block a key is in is now the answer to "do I need to restart?",
and it cannot drift the way a comment would: `reload_config` compares the incoming startup block
against the one the process booted with and returns an error if it differs.

`runtime` is honest about its reach. `CollectionManager` re-reads the config when it opens a
collection, so a reload applies to collections opened after it and to each request that reads a
value on its way through — not to collections already in memory.

Three rules keep the surface legible:

- **One place per setting.** `runtime.search` holds `ef` and `nprobe`, not each `IndexConfig`
  variant; `runtime.execution` chooses a strategy, not a `mode` field per index family;
  `startup.threads` is the only thread count. A knob that can be spelled two ways is a bug.
- **Nothing is silently ignored.** `deny_unknown_fields` throughout, so a typo or a key in the
  wrong block fails at startup naming it. Settings whose code isn't written yet — `runtime.inference`,
  `startup.embedding.provider: piramid` — exist so the shape is fixed before the work lands, and
  `validate` refuses them rather than accepting a value nothing reads.
- **The example is tested.** `config.example.yaml` is the whole surface at its defaults, with tests
  asserting it deserializes to exactly `Config::default()` and that every key appears in it.

Environment variables are overrides only, spelled mechanically from the path:
`runtime.cache.max_bytes` is `PIRAMID__RUNTIME__CACHE__MAX_BYTES`, parsed as YAML so `8`, `true`
and `null` mean what they do in the file. There is no table of names to keep in sync.
`OPENAI_API_KEY` is the one environment-only setting, so a key never lands in a file that gets
shared, and the support bundle redacts it.

## Errors

`core` is transport-agnostic. `PiramidError::kind()` returns an `ErrorKind` — `NotFound`,
`Conflict`, `Upstream`, `Internal`, and so on — with no notion of a status code.
`server::http::ApiError` is a newtype in the transport layer that maps a kind onto an HTTP status
and renders JSON.

Handlers keep `?` because `ApiError` converts from anything that converts into `PiramidError`, so a
handler returns `ApiResult<T>` while everything below returns `piramid_core::Result`. It also means
the orphan rule isn't a problem, since the `IntoResponse` impl is on a local newtype.

## Invariants

1. `compute` and `gpu` depend on nothing in the workspace.
2. No library crate calls `std::process::exit`. Configuration loading returns a `Result`.
3. `core` never names an HTTP type.
4. Vendor SDK types, `cudarc` and `candle`, never escape their backend module.
5. `unsafe` appears at four audited sites, each with a `// SAFETY:` comment.
6. Cache and index are rebuildable from the record store.
7. Retrieval works with no model loaded, and `model` depends on nothing in the retrieval
   stack. `model::fusion` holds only the trait; a strategy that queries an index is a
   separate crate depending on both. Enforced by `scripts/check-deps.sh`.
8. Default builds are CPU-only and need no vendor toolchain.
9. Telemetry speaks protocols, not products. Nothing is sent to this project under any
   configuration.

## Where new code goes

1. HTTP-specific goes in `apps/engine/serving/src/http`.
2. Something that coordinates a user-facing operation goes in `apps/engine/serving/src/services`.
3. Something that changes one collection's state goes in `apps/engine/database`.
4. Bytes, mmap, WAL, and sidecars go in `apps/engine/database`.
5. An ANN implementation detail goes in `apps/engine/database/src/index`.
6. Distance math or backend dispatch goes in `apps/engine/hardware`.
7. Device memory, streams, and kernels go in `apps/engine/hardware/src/gpu`.
8. Model execution goes in `apps/engine/model`.
9. Retrieval inside the forward pass goes in `apps/engine/model/src/fusion`.
10. Shared vocabulary — error, config, metadata — goes in `apps/engine/core`.
11. A deployable, a site, or a client library goes in `apps/`.

If a change touches three or more crates, start at the service boundary and make the data flow
explicit before writing anything.
