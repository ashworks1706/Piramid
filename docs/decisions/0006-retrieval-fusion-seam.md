# 0006 — Commit to the fusion seam, not to RETRO's mechanism

**Context.** The project's thesis is that retrieved documents should enter a model through
cross-attention during the forward pass rather than being pasted into the prompt. The reference
point is DeepMind's RETRO ([2112.04426](https://arxiv.org/abs/2112.04426)): a 7.5B model matching
GPT-3 on the Pile with a 2T-token retrieval database, via chunked cross-attention every 64 tokens.

Before building toward that specific mechanism, the evidence was checked. It is mixed, and three
findings cut against a literal RETRO implementation:

- **The encoder may be ablatable.** NVIDIA's InstructRetro
  ([2310.07713](https://arxiv.org/abs/2310.07713)), the largest retrieval-pretrained LLM at 48B,
  reports that removing the retrieval encoder and using the decoder backbone alone gives
  comparable results. The gains came from retrieval-augmented *pretraining* improving the decoder,
  not from retrieval in the attention loop at inference — which is precisely the mechanism in
  question.
- **Gains may be token overlap, not generalization.**
  [2302.12128](https://arxiv.org/pdf/2302.12128) finds RETRO's improvements largely track token
  duplication between test data and the retrieval store, and largely disappear where overlap is
  low.
- **The benefit saturates early.** *To Memorize or to Retrieve*
  ([2604.00715](https://arxiv.org/abs/2604.00715)), OLMo-2 from 30M to 3B over 100B tokens, finds a
  median 91% of the maximum retrieval gain realized at roughly one retrieval token per model
  parameter, and that the effect is task-, regime-, and metric-dependent.

Against that, the underlying premise is being validated at the frontier by a different mechanism:
DeepSeek's Engram (January 2026) argues transformers lack a native knowledge-lookup primitive and
adds one inside the forward pass — via deterministic hashed n-gram lookup fused into the residual
stream, not ANN plus cross-attention, and trained in at pretraining time.

**Decision.** Commit to the *seam*, not the mechanism.

`inference::fusion::RetrievalHook` says when retrieval may occur (`FusionPoint`) and what it may
touch (`ForwardContext`). It says nothing about how retrieved data is combined. Chunked
cross-attention, residual-stream gating, hashed n-gram lookup, and learned index routing are all
implementations of one trait.

The trait is defined before any code can call it, because a forward-pass driver written without the
seam is very hard to retrofit with one and a driver written with it costs nothing.
`NoopRetrievalHook` is both the default and the control arm for any benchmark claiming a strategy
helps.

**Consequences.** The engineering is not blocked on the research question. The infrastructure
(device runtime, contiguous layout, kernel dispatch, index) is valuable regardless of which fusion
mechanism wins, and none of it has to be rewritten if the answer changes.

**Not decided — deliberately.** Which mechanism. Before building one, the cheap experiment is a
throwaway Python retrofit of a small model measuring fused-vs-prompt-stuffed on a real target
corpus. Weeks, not months, and it decides whether the Rust engine is worth building for this. Note
that `lucidrains/RETRO-pytorch`'s train/test split leakage is the same confound as finding two
above; the JetBrains-Research fork fixes it and is the better starting point.

**What would falsify the thesis.** Fused retrieval failing to beat prompt-stuffing at equal token
budget on a corpus with low train/test overlap. That is the experiment to run first.
