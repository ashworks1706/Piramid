# 0010 — One name, one meaning

**Context.** After several rounds of moving crates, an audit of every duplicated file and
directory name in `apps/engine` turned up six places where the same word meant different things.
A tree can only tell you where to work if a name means one thing.

The worst offenders, each a genuine ambiguity rather than parallel naming:

- `inference/src/retrieval` collided with the `retrieval/` crate group — created by
  [0009](0009-retrieval-hook-is-a-module.md) an hour earlier.
- `inference/src/runtime` and `service/server/src/runtime`: one is a forward-pass driver, the
  other is process-wide shared state. Nothing connects the two meanings.
- `storage/src/metadata.rs` held `CollectionMetadata` — a collection's name, dimensionality, and
  counts. `core/src/metadata` holds `Metadata`, the key-value payload on a single document. Two
  unrelated concepts, one word.
- `storage/src/persistence/index.rs` held the *byte-offset* map from document id to file range.
  Reading it next to `retrieval/index` suggested an ANN index; it is the opposite end of the
  system.
- `services/search.rs` (parsing and DTO mapping) sat beside `services/vector/search.rs` (the
  endpoints), indistinguishable by name.
- `services/metadata.rs` was a third thing called metadata: JSON conversion helpers.

**Decision.** Rename for meaning, not for symmetry.

| Was | Now | Why |
|---|---|---|
| `inference::retrieval` | `inference::augment` | "retrieval-augmented" is the domain term, and it frees `retrieval` for the crate group |
| `inference::runtime` | `inference::forward` | It drives the forward pass; `runtime` already means process state in `server` |
| `storage::metadata` | `storage::manifest` | It describes a collection, not a document |
| `storage::persistence::index` | `storage::persistence::offsets` | It maps ids to byte ranges, not vectors to neighbours |
| `services::search` + `services::metadata` | `services::convert` | Both are DTO-to-domain conversion; neither is a service |

Remaining repeats are parallel naming inside a consistent scheme — `config/{index,search,cache}.rs`,
`error/{index,storage}.rs`, `{flat,hnsw,ivf}/{index,config}.rs` — where the repeated word means the
same thing at each site. Those stay.

Tests also stopped writing to `.piramid/tests` relative to the crate directory, which had scattered
four gitignored data directories through the source tree. They use `CARGO_TARGET_TMPDIR` now, which
is Cargo's own per-package scratch space and does not depend on how deep a crate sits.

**Consequences.** No behaviour changed and no public type was renamed; `CollectionMetadata` keeps
its name, since it is unambiguous once qualified. Every duplicated name left in the tree is now
either the same concept at different layers or a deliberate parallel.

**Not decided.** Whether `CollectionMetadata` should become `CollectionManifest` to match its
module. It is public API, the module name already disambiguates, and pre-1.0 churn on a type that
appears in user code is not obviously worth it.
