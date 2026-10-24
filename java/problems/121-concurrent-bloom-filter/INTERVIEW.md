# Interview mode — 121 Concurrent Bloom Filter

**Time limit: 60 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 5 min |
| 2 | Identify use cases / flows (add new, add duplicate, probe, estimate, clear, concurrent add + probe) | 4 min |
| 3 | Design the public API behind the entry point | 5 min |
| 4 | Design the model: what is stored, how an item maps to positions, who owns the state, invariants, what makes a concurrent add safe | 9 min |
| 5 | Implement single-threaded behaviour (sizing, add/probe, estimate, clear); wire it in | 18 min |
| 6 | Make it thread-safe; run the concurrent tests; fix | 11 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 3 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 5 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Java 21, Maven, JUnit 5. Prefer `final`/records for value types.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time? Which operations can lose an update, and why can't they here?
- What is the complexity of each operation (including `setBits()` and `approximateSize()`)?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
