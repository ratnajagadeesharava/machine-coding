# Interview mode — 143 Excel Cell Values and Sum Formulas (Rust)

**Time limit: 90 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 7 min |
| 2 | Identify use cases / flows (set a number, set a formula, change a cell in the middle of a chain, break a cycle) | 4 min |
| 3 | Design the public API (what does a caller do?) | 4 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 12 min |
| 5 | Implement the core flow; wire it into the test entry point | 35 min |
| 6 | Run tests; fix; handle edge cases, errors and cycles | 16 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 5 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 7 min |

## Rules

1. You create the structs/enums/traits. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Rust (stable), cargo. No `unsafe`.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each type? Why does each responsibility live where it does?
- Why is each trait necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time?
- What is the complexity of each operation, in the number of cells, the size of ranges and the length of chains?
- What would break first on the 2,600-cell chain, and how do I know it does not?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
