# Interview mode — 21 In-Memory Key-Value Store (Java)

**Time limit: 90 minutes** · Difficulty: Medium-Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 7 min |
| 2 | Identify use cases / flows | 4 min |
| 3 | Design the public API (what does a caller do?) | 7 min |
| 4 | Design the domain model: concepts, owners of state, relationships, invariants | 11 min |
| 5 | Implement the core flow; wire it into the test entry point | 31 min |
| 6 | Run tests; fix; handle edge cases and invalid operations | 14 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 6 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 9 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Java 21, Maven, JUnit 5. Prefer `final`/records for value types, `java.util.concurrent` over hand-rolled threading.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- Why is each interface necessary? Could it be deleted tomorrow?
- What are the invariants, and which code enforces each one?
- What happens if two operations happen at the same time?
- What is the complexity of each operation?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
