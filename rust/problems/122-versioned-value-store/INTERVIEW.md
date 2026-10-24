# Interview mode — 122 Versioned Value Store (Rust)

**Time limit: 75 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 6 min |
| 2 | Identify use cases / flows (write, read as of a version, compare-and-set, snapshot, compact) | 3 min |
| 3 | Design the public API (what does a caller do?) | 5 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 10 min |
| 5 | Implement the core flow (versions, put/get/getAt/history/delete); wire it into the test entry point | 22 min |
| 6 | Add compare-and-set, putAll, snapshots, compaction; run tests; fix | 17 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation, check the thread-safety story | 4 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 8 min |

## Rules

1. You create the structs/enums/traits. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: stable Rust, cargo. Prefer plain structs and enums; share state only as far as the thread-safety story needs.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each type? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants (gap-free versions, history ordered by version, snapshots never change), and which code enforces each one?
- What happens if two operations happen at the same time? Which operations must be atomic together, and why?
- What is the complexity of each operation, and of a compaction?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
