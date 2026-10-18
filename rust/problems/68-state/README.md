# 68 · State (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — one stub type, `Ticket`, with the full transition table in its doc comment; `TicketError` is provided. You add the state types and trait. |

> **Pattern:** *State* — let an object change its behaviour when its internal state changes, by delegating to a
> **state object**. Each state type knows only its own rules and which state comes next, so `Ticket` contains no big
> `match` over the state, and adding a state means adding a type.
>
> **Rust adaptation:** a common shape is `Box<dyn TicketState>` where each transition takes `self: Box<Self>` and
> returns the next boxed state (so a state can consume itself); shared data (agent, comments, reopen count, time
> resolved) lives in `Ticket` and is passed to the state methods.

## Problem

A help-desk ticket moves through `NEW → IN_PROGRESS ⇄ ON_HOLD → RESOLVED → CLOSED`. What you may do depends on the
state: you cannot resolve a ticket that nobody has picked up, and you cannot touch a closed one.

## Requirements

Implement the transition table in `Ticket`'s doc comment:

1. `assign(agent)`: NEW → IN_PROGRESS; in IN_PROGRESS or ON_HOLD it only changes the assigned agent.
2. `hold(reason)`: IN_PROGRESS → ON_HOLD, comment `HOLD: <reason>`. `resume()`: ON_HOLD → IN_PROGRESS.
3. `resolve(note)`: IN_PROGRESS → RESOLVED, comment `RESOLVED: <note>`, remembers the time (`Clock::now_millis`).
4. `reopen()`: RESOLVED → IN_PROGRESS, comment `REOPENED`; at most **twice** per ticket (then `Err(TicketError::ReopenLimit)`).
5. `close()`: RESOLVED → CLOSED. CLOSED is terminal: **everything** (including `comment`) is rejected.
6. `comment(text)` works in every state except CLOSED.
7. `auto_close_if_stale()`: a RESOLVED ticket resolved 72 hours or more ago becomes CLOSED and returns `true`; otherwise `false` and nothing changes.
8. A disallowed operation returns `Err(TicketError::Illegal("cannot <operation> a ticket in state <STATE>"))` (operations: assign, hold, resume, resolve, reopen, close, comment). Blank arguments are rejected first with `Err(InvalidArgument)`. A failed call changes nothing.

## Constraints (what the tests catch)

- Keep everything in `src/lib.rs`. The source must not contain `match self.state`, `match &self.state`, `match state`, `match self.status` or `match &self.status`, and must have at least **5** `impl <Trait> for <Type>` lines (one per state).

## Running

```bash
cargo test -p p68_state     # from rust/ — tests FAIL until you implement the stubs
cargo play run 68
```
