# Interview mode — 113 Multi-Module Logging Framework (Rust)

**Time limit: 90 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 8 min |
| 2 | Identify use cases / flows (log, fail, query, close) | 5 min |
| 3 | Design the public API (what does a caller do?) | 7 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants, thread ownership | 12 min |
| 5 | Implement the core flow; wire it into the test entry point | 32 min |
| 6 | Run tests; fix; handle edge cases, failures and shutdown | 14 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 5 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 7 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Rust (stable), cargo. Prefer small owned types; share state through `Arc` and justify every lock.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time? Which thread owns the sequence counter and the stored logs?
- What happens when a producer is waiting and `close()` is called? When the sink throws?
- What is the complexity of each operation?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
