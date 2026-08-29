# 0009 — The retrieval hook is a module, not a crate

Supersedes the `inference` split in [0008](0008-engine-and-apps.md). The rest of that record stands.

**Context.** [0008](0008-engine-and-apps.md) split `inference` into `piramid-fusion` (the
`RetrievalHook` trait, depending only on `core`) and `piramid-inference` (the forward-pass driver).
The stated reason was that a fusion *strategy* must query an index, so isolating the trait kept the
runtime free of the retrieval stack.

Reviewing it against how comparable Rust systems are cut turned up three problems.

**The name is taken, and by the adjacent field.** In the Rust ML ecosystem "fusion" means *kernel
fusion*: [`burn-fusion`](https://crates.io/crates/burn-fusion) is a kernel-fusion backend
decorator, and Candle ships `candle-flash-attn` for fused attention. A crate called
`piramid-fusion` sitting next to `piramid-gpu` reads as "fuses GPU kernels" to precisely the
readers most likely to look.

**The dependency argument did not hold.** The claim was that only a crate boundary could prevent
`inference → search`. It cannot: the boundary that matters is on the *strategy*, not the trait. A
strategy crate depending on `piramid-inference` and `piramid-search` keeps the direction correct
however the trait is packaged. `scripts/check-deps.sh` asserts `piramid-inference` has no
retrieval dependency, which is the actual invariant, and it holds either way.

**No precedent for a two-file trait crate.** Candle keeps `BackendDevice` and `BackendStorage` in
`candle-core` beside implementations. Burn's `Backend` trait lives in a substantial crate, not one
extracted for direction. Candle splits `candle-kernels` and `candle-flash-attn` out because their
*builds* differ — a CUDA toolchain — which is a real reason a crate boundary earns its cost.

**Decision.** Fold the trait back in as `piramid_inference::retrieval`, and rename `FusionPoint` to
`RetrievalPoint` and `on_fusion_point` to `on_retrieval_point`. The concept keeps the name
"retrieval fusion" in prose, where it is the literature's term and unambiguous; only the crate and
the API stop using a word that collides.

`scripts/check-deps.sh` keeps asserting that `piramid-inference` depends on nothing in the
retrieval stack. Verified by injecting `piramid-inference -> piramid-search` and watching it fail.

**Consequences.** Twelve crates become eleven. The invariant that made the split seem necessary is
unchanged and still enforced. Anyone reading `hardware/gpu` beside `inference/` no longer has to
work out which kind of fusion is meant.

**Not decided.** Whether kernels eventually become their own crate. Candle's precedent says yes
once the CUDA build diverges from the CPU one — a `.cu` toolchain is a build-system boundary, and
that is the kind that justifies a crate. `gpu/kernels/` is a module until then.
