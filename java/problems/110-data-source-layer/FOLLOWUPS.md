# Follow-ups — 110 Runtime Data Source Access Layer

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — A fourth source, without touching callers
A `CACHE` source is added: in-memory, holds at most 3 records, and when full a `put` of a new key drops the oldest record. Business code and the operations it calls must not change. List every place in your code that has to change and justify each one. Could someone outside your module register a source?

### V3 — Read-through fallback
Operations staff can mark one source as the fallback of another: a `get` that finds nothing in the current source asks the fallback and, on a hit, remembers the record in the current source. What do `size`, `remove` and the API length limit mean now? Which of your earlier guarantees weaken, and which do you keep?

### V4 — Switching with hand-over
`switchSource(name, copyExisting)` can copy the old source's records into the new one, each record going through the new source's own acceptance rules. What happens to a 40-character value moving into `API`, and to `"A"` and `"a"` moving from `DATABASE` into `API`? Is the switch still atomic for concurrent callers while it copies?

### V5 — Per-caller source and observability
Different tenants want different current sources at the same time, and operations wants a log of every switch (who, from, to) and a per-source call counter. How does this change who owns the current-source state, and how do you keep reads cheap while the counters are updated?

### V6 — Failures and health
A source can become unavailable (simulate it); calls on it fail while the others keep working, and the layer can switch to a healthy source automatically after 3 consecutive failures. Where does the failure policy live, and how do you test it deterministically without sleeping?
