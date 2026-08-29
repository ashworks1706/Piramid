# 0008 — `engine/` for the library tree, `apps/` for what we ship

Supersedes the folder naming in [0002](0002-workspace-crate-boundaries.md); the crate boundaries
and the dependency law it established are unchanged.

**Context.** [0002](0002-workspace-crate-boundaries.md) put every library crate in a flat
`crates/`. Two problems showed up once it existed.

`crates/` names Rust's compilation model, not the product. It tells a reader that these are Rust
libraries — true, and useless. Eleven sibling directories with no grouping also gave no sense of
which parts belong together: `compute` and `gpu` are one concern, `storage`/`index`/`search` are
another, and the tree said nothing about that.

Separately, `apps/` held only the CLI while `website/`, `blogs/`, and `sdk/` sat at the repo root,
which said the site and the clients were somehow not part of the project. And the website reached
outside its own directory for content (`path.join(process.cwd(), "..", "blogs")`), so it could not
be built or deployed standalone.

**Decision.** `engine/` for the library tree, grouped by subsystem; `apps/` for everything we ship.

```
engine/foundation/  core
engine/hardware/    compute  gpu
engine/retrieval/   storage  index  search  collections  embeddings
engine/inference/   fusion  runtime
engine/service/     server  observability
apps/               cli  website  sdk
```

"One binary" describes the artifact, not the tree. The engine is twelve crates across five
subsystems; `apps/cli` is what fuses them into one binary. Naming the library tree `engine/` makes
that relationship legible, and the product is literally called an inference engine.

`apps/` means "what we author and ship", which is why the SDKs live there despite being libraries
rather than deployables. The alternative — reserving `apps/` for deployables only and adding a
third top-level `sdks/` — splits hairs the repo does not need.

Subsystem groups are for navigation, not stratification. They deliberately do not line up with the
dependency order: `foundation/core` depends on `hardware/compute` for the `ExecutionMode` and
`Metric` types that configuration carries. The law in `scripts/check-deps.sh` is the authority on
direction; the folders are an index.

`inference` also split in two. `piramid-fusion` holds the `RetrievalHook` trait and depends only on
`core`; `piramid-inference` is the forward-pass driver and depends on `fusion` and `gpu`. A
concrete fusion strategy will be a third crate depending on both `fusion` and `search`. That keeps
the model runtime free of the retrieval stack, which is what makes "a collection stays queryable
with no model loaded" a structural fact rather than a convention. `check-deps.sh` enforces it.

**Consequences.** No crate names changed and no code changed — `use piramid_gpu::Device` is what it
always was. This was path movement plus manifest edits, which is exactly why it was worth doing
now rather than in six months.

Moving the website into `apps/` also forced its content in with it, which surfaced a live bug: blog
images were duplicated between `assets/blogs/` and `website/public/assets/blogs/`, the copies had
drifted, and the public copy was missing `lsm.png` entirely — so that image had not been rendering
on the site. There is now one copy.

The tree is three levels deep to a crate (`engine/retrieval/index`), which is one more than before.
That is the cost of the grouping and it is worth it at twelve crates; it would not be at four.

**Not decided.** Whether the SDKs survive at all. They are 11 and 7 lines, published under names
already claimed on npm and PyPI. Either they become real clients or they should be unpublished —
a stub under a name people can install is worse than no package.
