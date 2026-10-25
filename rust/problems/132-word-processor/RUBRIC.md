# Rubric — 132 Word Processor (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
| Performance and memory behaviour | 15% |

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

- Is formatting stored per range or as shared immutable values, rather than one independent object per character? Can the candidate say what a million identical characters cost?
- Is `runs()` canonical after every edit (insert inside a run, delete between equal runs, format that equalises neighbours)?
- Is the position-0 rule (default format) distinct from the "preceding character" rule, and is the empty document handled?
- Are `applyFormat` inputs validated completely before anything changes?
- Are `find` (overlapping) and `replaceAll` (non-overlapping, no rescan, first-character format) clearly separated, and are replacement offsets handled correctly when lengths differ?
- Does alignment follow the lines through inserts, deletes and replacements?
- Is one lock (or a clearly argued alternative) protecting text, formats and alignments together so readers never see a mixture?
