# Interview mode — 124 Kafka-Like Message Streaming Service (Rust)

**Time limit: 90 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 8 min |
| 2 | Identify use cases / flows (publish, join/leave, poll, commit, retention) | 5 min |
| 3 | Design the public API (what does a caller do?) | 7 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 12 min |
| 5 | Implement the core flow; wire it into the test entry point | 33 min |
| 6 | Run tests; fix; handle edge cases and invalid operations | 14 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 5 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 6 min |

## Rules

1. You create the types, traits and enums. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Rust (stable), cargo. Prefer enums and small structs; keep ownership of each piece of state obvious.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each type? Why does each responsibility live where it does?
- Why is each trait necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time?
- What is the complexity of each operation?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
