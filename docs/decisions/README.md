# Decisions

Piramid's design decisions in one file, compressed. The eleven separate records that used to live
here were written while the shape was still moving; the tree they describe no longer exists in that
form, so they were folded into the summary below and deleted.

This is the starting point. From here, a change that moves a boundary or forecloses an option gets
a new numbered record beside this file.

## What was decided, and what still holds

**One binary, layers as crates.** Piramid ships one executable. The crates enforce direction:
Rust checks a dependency rule across crate boundaries and never inside one. `apps/cli` links them
into the single binary.

**`apps/engine/` for the library tree, `apps/` for what we ship.** `crates/` names Rust's
compilation model rather than the product. Everything authored lives under `apps/`, including the
SDKs. Reaching a crate is two levels, with no grouping folders — a folder hierarchy that does not
match the dependency rule is a second model to keep in your head.

**Five crates.** `hardware → core → database → serving`, with `model` depending on core and
hardware only.

- `hardware` depends on nothing in the workspace, so kernels can be benchmarked alone and `model`
  can get a device without reaching through retrieval math.
- `core` is shared vocabulary: errors, the whole configuration surface, document metadata and its
  filters, validation, self-measurement and its export.
- `database` is bytes, the structures that find them, and the collection composing both. Storage,
  index, search and collections share a crate because separating them is a cycle: a collection is
  built on search, search on storage, and storage holds a collection's bytes.
- `model` never depends on `database`, which is what keeps a collection queryable with no model
  loaded. `scripts/check-deps.sh` fails the build on that edge.
- `serving` is the only crate that knows HTTP exists.

**Strategy-first compute dispatch.** One file per strategy under `compute/strategies/`, each
implementing the whole `DistanceKernels` trait, one arm in the registry. Adding hardware is a new
file, not a new match arm in every kernel. "Backend" means the vendor layer and appears only in
`gpu/backends/`.

**`gpu` owns the device, `compute` owns the math.** Contexts, buffers, streams and modules are one
concern; what cosine means is another. Both live in `hardware` but neither imports the other's
vocabulary, because two subsystems need a device and neither should depend on the other to
allocate memory.

**Contiguous vector layout behind an optional seam.** `VectorReader::as_slab()` returns a
row-major buffer when a reader has one and `gather_into()` is the fallback. Both have defaults, so
a new reader costs nothing and the device-upload path can arrive later without touching callers.
Batch kernels take a slab and a caller-owned `out` for the same reason: that shape uploads in one
copy, and a slice of `Vec`s cannot.

**Commit to the fusion seam, not to a mechanism.** `model::fusion::RetrievalHook` says when
retrieval may occur and what it may touch, and nothing about how retrieved data is combined.
Chunked cross-attention, residual-stream gating and learned routing are all implementations of the
same trait. `launch` is separate from `join` so retrieval can overlap the pass it runs beside, and
`NoopRetrievalHook` is the control arm any benchmark is measured against.

**Errors carry a kind, not a status code.** `PiramidError` exposes an `ErrorKind`; the mapping onto
HTTP lives in `serving::http::ApiError`. Every error crossing a crate boundary must be reachable
from `PiramidError`, so nothing has to be stringified to travel.

**One name, one meaning.** Before naming a module, check the word is not already used for something
else. Repeating a word is fine when it means the same thing at each layer and a bug when it does
not. Folders are named for what they hold, never for a Rust construct — no `types/`, no `utils/`.

**Open standards only.** Prometheus and OTLP are protocols; a vendor's product is not. Telemetry
points at an endpoint the operator supplies and integrates with nothing by name.

**Nothing is silently ignored.** A setting the build cannot honour is a startup error naming the
key, not a value quietly doing nothing. This is the rule the tree has broken most often: five
settings once shipped with no reader, one of them `wal.sync_on_write`, which the docs described as
the durability knob while the WAL never called `fsync`.
