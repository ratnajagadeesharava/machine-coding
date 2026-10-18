# 68 · State (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — one stub class, `Ticket`, with the full transition table in its Javadoc. You add the state classes. |

> **Pattern:** *State* — let an object change its behaviour when its internal state changes, by delegating to a
> **state object**. Each state class knows only its own rules and which state comes next, so `Ticket` contains no
> big `switch`/`if` over the state, and adding a state means adding a class.

## Problem

A help-desk ticket moves through `NEW → IN_PROGRESS ⇄ ON_HOLD → RESOLVED → CLOSED`. What you may do depends on the
state: you cannot resolve a ticket that nobody has picked up, and you cannot touch a closed one. A first version used
one `Ticket` class with `if (status == ...)` in every method; every new rule broke another.

## Requirements

Implement the transition table in `Ticket`'s Javadoc:

1. `assign(agent)`: NEW → IN_PROGRESS; in IN_PROGRESS or ON_HOLD it only changes the assigned agent.
2. `hold(reason)`: IN_PROGRESS → ON_HOLD, comment `HOLD: <reason>`. `resume()`: ON_HOLD → IN_PROGRESS.
3. `resolve(note)`: IN_PROGRESS → RESOLVED, comment `RESOLVED: <note>`, remembers the time.
4. `reopen()`: RESOLVED → IN_PROGRESS, comment `REOPENED`; at most **twice** per ticket (the third attempt is an `IllegalStateException`).
5. `close()`: RESOLVED → CLOSED. CLOSED is terminal: **everything** (including `comment`) is rejected.
6. `comment(text)` works in every state except CLOSED.
7. `autoCloseIfStale()`: a RESOLVED ticket resolved 72 hours or more ago (per the injected `Clock`) becomes CLOSED and returns `true`; otherwise `false` and nothing changes.
8. An operation that is not allowed in the current state throws `IllegalStateException("cannot <operation> a ticket in state <STATE>")` (operation names: assign, hold, resume, resolve, reopen, close, comment). Blank arguments are rejected first with `IllegalArgumentException`. A failed call changes nothing.

## Constraints (what the tests catch)

- The `solution` package must have **at least 6** `.java` files (the ticket plus one per state) and none of the code may contain `switch`, `state ==`, `state.equals`, `.ordinal()` or `getClass()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
