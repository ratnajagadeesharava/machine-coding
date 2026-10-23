# Interview mode — 118 Search Result Cache (Rust)

**Time limit: 75 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 6 min |
| 2 | Identify use cases / flows (hit, miss, expired, stale, in flight, failure) | 4 min |
| 3 | Design the public API behind the entry point | 6 min |
| 4 | Design the model: entries, recency, in-flight calls, owners of state, invariants, lock scope | 10 min |
| 5 | Implement single-threaded behaviour (normalise, TTL, version, LRU, stats); wire it in | 22 min |
| 6 | Add coalescing and failure handling; run tests; fix | 14 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 4 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 9 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Rust (stable), cargo. Prefer enums and small structs over trait objects unless you need polymorphism.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time? Which lock is held while the backend runs?
- What is the complexity of each operation (including `size()`)?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
