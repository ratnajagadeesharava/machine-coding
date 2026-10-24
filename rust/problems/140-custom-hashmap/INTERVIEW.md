# Interview mode — 140 Custom HashMap (Rust)

**Time limit: 60 minutes** · Difficulty: Medium

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 5 min |
| 2 | Identify the operations and what each must cost | 3 min |
| 3 | Design the public API and the state you will hold | 5 min |
| 4 | Decide how entries are stored, how growth works, how order is kept; state invariants | 7 min |
| 5 | Implement the core operations; wire them into the test entry point | 22 min |
| 6 | Run tests; fix; handle edge cases (collisions, negative hashes, invalid input) | 8 min |
| 7 | Make each operation atomic; refactor; remove unjustified abstractions | 5 min |
| 8 | Explain trade-offs aloud (or write 5 sentences) | 5 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not store the entries in a standard-library map or set (`HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, ...). The tests cannot check this; the exercise is pointless if you do.
3. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
4. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
5. Language: stable Rust, `cargo test`. Prefer plain structs and enums; no `unsafe` unless you can justify it.
6. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- What are the invariants, and which code enforces each one?
- What is the complexity of each operation, on average and in the worst case? What makes the worst case happen?
- When exactly does the table grow, and what does a resize touch?
- How many times does my code call the hasher per operation, and why?
- What happens if two operations happen at the same time?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
