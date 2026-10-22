# Rubric — 119 Mini RAG Retrieval Component (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities, swappable parts) | 25% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
| Performance | 10% |

Score each area 0–4 (0 = absent, 2 = acceptable, 4 = would hire on this alone), multiply by weight.

## Questions for every problem

**Design**
- Did the candidate identify the correct domain concepts?
- Are responsibilities well separated; does each piece of state have one clear owner?
- Are abstractions justified by a requirement? (Not: "did they use pattern X?")
- Is the API clean, hard to misuse, and free of leaked internals?
- Can the requirements evolve (see `FOLLOWUPS.md`) without a rewrite?
- Is the implementation unnecessarily complicated?

**Correctness & quality**
- Are invariants preserved across every operation, including failed ones (no half-applied changes)?
- Are edge cases (empty, boundary, repeated, invalid operations) handled deliberately?
- Can components be tested independently of the whole system?

**Concurrency & performance**
- Is concurrency safe (or explicitly and correctly declared out of scope)? Is the locking granularity reasoned about?
- Are performance characteristics reasonable for the stated constraints?

**Language-specific**
- Is ownership clear — who owns which state, and where are borrows vs clones used?
- Are traits introduced only where polymorphism is needed (enum vs trait object vs generics)?
- Is shared state (`Arc`/`Mutex`/`RwLock`/atomics/channels) used only where actually required?
- Does the code pass `cargo fmt --check` and `cargo clippy` cleanly?

## Problem-specific

- Is chunking exact (start positions, overlap, no redundant trailing chunk, single-space joined text) and validated without changing state on error?
- Is the part that decides what to embed, the part that stores vectors, and the part that assembles prompts separated so that the embedder and the vector store can each be replaced without touching the rest?
- Does the engine keep the supplied vector store exactly in step with the documents (replace removes stale chunks, delete removes all and only that document's chunks)?
- Are ties, zero vectors, `k` larger than the index and string-ordered chunk ids handled as specified?
- Is the budget rule exact (separators and rank prefix counted, whole chunks, stop at the first that does not fit, `(none)`)?
- Is the embedder called outside any lock that blocks readers, and is a replace atomic for readers and all-or-nothing when the embedder fails?
- Could a new ranking step, a new chunker or a new prompt template be added by adding code rather than editing the engine?
