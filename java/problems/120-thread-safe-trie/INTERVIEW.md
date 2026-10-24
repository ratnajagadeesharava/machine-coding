# Interview mode — 120 Thread-Safe Trie (Java)

**Time limit: 75 minutes** · Difficulty: Hard

Set a timer. Do not open `INTERVIEWER_NOTES.md`. Open `HINTS.md` only if you are stuck (one level at a time). Open `FOLLOWUPS.md` only when told (or when you finish early) — one version at a time.

## Phases (suggested time boxes)

| # | Phase | Time |
|---|---|---|
| 1 | Read the requirements; list assumptions and open questions (write them down) | 5 min |
| 2 | Identify use cases / flows (insert, query by prefix, delete, concurrent mix) | 3 min |
| 3 | Design the public API (what does a caller do?) and say what the hot path is | 5 min |
| 4 | Design the domain model: state, who owns it, invariants, and what "atomic" means for each call | 10 min |
| 5 | Implement the single-threaded behaviour first; wire it into the test entry point | 20 min |
| 6 | Make it thread-safe, state the guarantee per operation; run the concurrency tests; fix; handle edge cases | 20 min |
| 7 | Refactor: remove unjustified abstractions, tighten encapsulation | 4 min |
| 8 | Explain trade-offs aloud (or write 5 sentences): complexity, locking granularity, memory | 8 min |

## Rules

1. You create the classes/interfaces/enums/value objects. The repository will not.
2. Do not add an abstraction until a requirement justifies it. Be ready to say which requirement.
3. Keep a `NOTES.md` of assumptions you made where the requirements were silent.
4. Language: Java 21, Maven, JUnit 5. Prefer `final`/records for value types.
5. Tests are behavioral. If a test fails, your design or understanding is wrong — not the test (unless you can point at a contradiction with the README).

## Self-review (answer in writing before looking at anything else)

- Why did I create each class? Why does each responsibility live where it does?
- What are the invariants, and which code enforces each one?
- For each operation: at which instant does it take effect, and what stops another thread from seeing it half-done?
- Can a `wordsWithPrefix` call return a word that was never in the set, a duplicate, or an unsorted list?
- What is the complexity of each operation in terms of word length, matches and `limit`? What does deleting a word free?
- Where is the contention if 16 threads insert different words? If they all insert the same word?
- How much memory does one word cost? What would 10 million words cost?
- What did I intentionally NOT abstract? Where might I have over-engineered?
- What happens when requirement V2 (see `FOLLOWUPS.md`) arrives — what changes, what stays?
