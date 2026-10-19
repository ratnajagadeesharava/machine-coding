# Interviewer notes — 80 Parallel Word Count

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A word at the very start / end of a chunk.
- Documents that are empty or contain only separators.
- Fewer documents than workers (idle workers).
- Upper/lower case of the same word in different chunks.
- `é` and other non-ASCII characters splitting a word.

## Likely interviewer questions
- Why are per-thread maps usually faster than one shared concurrent map?
- What does the final merge cost, and can it become the bottleneck? How would you parallelise it?
- How do you split one very large document safely?
- Is `ConcurrentHashMap<String, Integer>` with `get`/`put` correct? With `merge`?
- How would the design change for a streaming source?

## Invariants
- Sum over the result equals the number of words in the input.
- Each word occurrence is counted by exactly one worker.
- Chunk boundaries fall on separators.

## Common mistakes
- `HashMap` shared between threads (lost updates, even infinite loops/corruption).
- `get` then `put` on a concurrent map (not atomic).
- Cutting a word in half at a chunk boundary or counting it twice.
- Lower-casing after merging so 'The' and 'the' are counted separately in different workers.
- Spawning more threads than `parallelism`.

## Over-engineering traps
- A pluggable tokenizer/aggregator/reducer pipeline.
- Reaching for parallel streams without understanding what they merge with.

## Concurrency discussion
- Map-reduce in miniature: shard, count locally, reduce. The reduce step is the sequential bottleneck; a tree reduction or partitioning by word hash avoids it.
- A shared concurrent map is simpler but suffers from contention on hot keys and cache-line ping-pong.

## Trade-off questions to ask when the candidate is done
- Why thread-local maps and not a concurrent map? Show the workload where the concurrent map wins.
- What happens with a Zipfian vocabulary (a few words dominate)?
- How would you test the split-boundary logic specifically?
- How would you do the merge lock-free / in parallel?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
