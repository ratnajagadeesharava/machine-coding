# Interviewer notes — 79 Parallel Merge Sort

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Length 0 and 1; two elements.
- `parallelism` larger than the number of elements.
- Merge with `MIN_VALUE`/`MAX_VALUE` (overflow in comparisons).
- Scratch buffer indexed with the wrong offset for the right half.
- Using more threads than allowed by recursing without a budget.

## Likely interviewer questions
- How do you choose the threshold? What happens if it is 1? If it is the array length?
- Why do no locks protect the array, and which synchronisation point makes the writes visible to the merger?
- What limits the speed-up (Amdahl's law)? What is the sequential part here?
- Why can a fixed-size pool deadlock on recursive fork/join, and how does work stealing avoid it?
- What does memory bandwidth do to the speed-up?

## Invariants
- Ranges handled concurrently are disjoint.
- A merge starts only after both of its inputs are final.
- Live worker threads never exceed `parallelism`.

## Common mistakes
- Spawning a thread per split down to size 1 (thousands of threads, slower than sequential).
- Merging before joining; or joining on a thread but not on the other half.
- Overflow in `a - b` comparisons; off-by-one at `mid`.
- Allocating a scratch array in every merge.
- Recursing on a bounded pool and blocking all workers in `join`.

## Over-engineering traps
- A generic `Task` / `Splitter` / `Merger` framework.
- Parallelising the base case with parallel streams as well.

## Concurrency discussion
- Speed-up is limited by the sequential merge and memory bandwidth; a parallel merge or a parallel sample sort scale better.
- Thread-per-split vs pool vs fork-join: ownership of threads, exception propagation and cancellation differ.

## Trade-off questions to ask when the candidate is done
- Why this threshold and this pool? How would you tune it?
- What happens with 64 threads on 4 cores?
- How would you test the thread bound and the speed-up reliably on a noisy CI machine?
- Could this be done lock-free / wait-free? (it already is lock-free: where is the only blocking point?)

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
