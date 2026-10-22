# Interview mode — 115 Autocomplete / Typeahead Service (Rust)

**Time limit: 75 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 5 min |
| 2 | Identify use cases / flows (record, suggest, personalised suggest, switch ranking) | 3 min |
| 3 | Design the public API (what does a caller do?) and say what the hot path is | 5 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 10 min |
| 5 | Implement the core flow (frequency first); wire it into the test entry point | 25 min |
| 6 | Add recency and personalisation; run tests; fix; handle edge cases and invalid operations | 15 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 4 min |
| 8 | Explain trade-offs aloud (or write 5 sentences): complexity, concurrency, memory | 8 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: stable Rust, `cargo test`. Prefer plain structs and enums; add traits only where you need polymorphism.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time? Can a reader see a half-updated ranking?
- What is the complexity of `record` and of `suggest`, in terms of prefix length, number of matches and k?
- How much memory does one term cost? What would 10 million terms cost?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
