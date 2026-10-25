# Interview mode — 130 Ludo (Rust)

**Time limit: 90 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 8 min |
| 2 | Identify use cases / flows (leave the yard, move, capture, extra turn, finish, invalid calls) | 4 min |
| 3 | Design the public API (what does a caller do?) | 5 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 12 min |
| 5 | Implement the core flow (turn handling, legal moves, move, capture); wire it into the test entry point | 30 min |
| 6 | Run tests; fix; handle edge cases and invalid operations | 17 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 6 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 8 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Rust (stable), cargo. Prefer small structs and enums with private fields.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- Where does "whose turn is it, and what may they do now" live, and how do invalid calls get rejected?
- How do I convert a (colour, progress) pair into a board cell, and why is that the only place that knows the offsets?
- What is the cost of one move?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
