# 68 · State (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 60 min | Level 2 — one stub class, `Ticket`, with the transition table in its Javadoc. You write the rest. |

## Problem

A help-desk ticket moves through `NEW → IN_PROGRESS ⇄ ON_HOLD → RESOLVED → CLOSED`. What you may do depends on the
state: you cannot resolve a ticket that nobody has picked up, and you cannot touch a closed one. A first version used
one `Ticket` class with `if (status == ...)` in every method; every new rule broke another.

**What you are building:** a support ticket for a help desk. A customer problem arrives as a NEW ticket; an agent
takes it (IN_PROGRESS), may park it while waiting for the customer (ON_HOLD), resolves it, and later it is closed.
A resolved ticket can be reopened (at most twice) if the fix did not work, and a resolved ticket nobody reopens is
closed automatically after 72 hours. Every operation is only legal in certain states, and a ticket keeps an ordered
list of comments describing what happened.

### Glossary

- **State**: the ticket's stage in its life, reported as the string `"NEW"`, `"IN_PROGRESS"`, `"ON_HOLD"`, `"RESOLVED"` or `"CLOSED"`.
- **Terminal**: a state nothing can leave. CLOSED is terminal.
- **Blank**: `null`, empty, or only whitespace.
- **Clock**: the `java.time.Clock` passed to the constructor; it is the only source of "now" (tests move it by hand).

## Requirements

`new Ticket(id, clock)` creates a ticket in state NEW, with no assigned agent and no comments (blank id → `IllegalArgumentException`). `id()`, `state()`, `assignedTo()` (`Optional<String>`, empty until assigned) and `comments()` (unmodifiable list, oldest first) read it.

Transition table (what each operation does in each state):

| State | assign | hold | resume | resolve | reopen | close | comment |
|---|---|---|---|---|---|---|---|
| NEW | -> IN_PROGRESS | illegal | illegal | illegal | illegal | illegal | ok |
| IN_PROGRESS | only changes agent | -> ON_HOLD | illegal | -> RESOLVED | illegal | illegal | ok |
| ON_HOLD | only changes agent | illegal | -> IN_PROGRESS | illegal | illegal | illegal | ok |
| RESOLVED | illegal | illegal | illegal | illegal | -> IN_PROGRESS (max 2 times) | -> CLOSED | ok |
| CLOSED | illegal | illegal | illegal | illegal | illegal | illegal | illegal |

1. `assign(agent)`: NEW → IN_PROGRESS and records the agent; in IN_PROGRESS or ON_HOLD it only changes the assigned agent (the state stays).
2. `hold(reason)`: IN_PROGRESS → ON_HOLD, adds the comment `HOLD: <reason>`. `resume()`: ON_HOLD → IN_PROGRESS.
3. `resolve(note)`: IN_PROGRESS → RESOLVED, adds the comment `RESOLVED: <note>`, remembers the time (from the clock).
4. `reopen()`: RESOLVED → IN_PROGRESS, adds the comment `REOPENED`; at most **twice** per ticket (the third attempt is an `IllegalStateException` and the ticket stays RESOLVED).
5. `close()`: RESOLVED → CLOSED. CLOSED is terminal: **everything** (including `comment`) is rejected.
6. `comment(text)` appends `text` exactly as given and works in every state except CLOSED.
7. `autoCloseIfStale()`: a RESOLVED ticket resolved 72 hours or more ago (per the injected `Clock`; exactly 72 h counts, 71 h 59 min does not) becomes CLOSED and returns `true`; in any other situation (other states, not yet 72 h, already closed) it returns `false` and changes nothing. After a reopen and a new resolve, the 72 hours count from the latest resolve.
8. An operation that is not allowed in the current state throws `IllegalStateException("cannot <operation> a ticket in state <STATE>")` (operation names: assign, hold, resume, resolve, reopen, close, comment). Blank arguments are rejected first with `IllegalArgumentException` (so a blank argument in an illegal state is still an `IllegalArgumentException`). A failed call changes nothing.

## Examples

Happy path:

```
t = new Ticket("T-1", clock)                 state NEW, assignedTo empty, comments []
t.assign("ann")                              IN_PROGRESS, assignedTo "ann"
t.hold("waiting for customer")               ON_HOLD
t.resume()                                   IN_PROGRESS
t.resolve("restarted the server")            RESOLVED
t.close()                                    CLOSED
t.comments()  -> ["HOLD: waiting for customer", "RESOLVED: restarted the server"]
```

Reopen limit:

```
RESOLVED ticket (comment "RESOLVED: fixed")
reopen(); resolve("again"); reopen(); resolve("and again")
reopen()  -> throws IllegalStateException; state stays RESOLVED
comments  -> ["RESOLVED: fixed", "REOPENED", "RESOLVED: again", "REOPENED", "RESOLVED: and again"]
```

Illegal operations (messages must match exactly):

```
new Ticket(..).resolve("x")        -> IllegalStateException "cannot resolve a ticket in state NEW"
IN_PROGRESS ticket .resume()       -> "cannot resume a ticket in state IN_PROGRESS"
RESOLVED ticket .assign("x")       -> "cannot assign a ticket in state RESOLVED"
CLOSED ticket .comment("late")     -> "cannot comment a ticket in state CLOSED"
```

Auto-close (ticket resolved at 09:00, clock moved by hand):

```
clock = 09:00 + 71h59m   autoCloseIfStale() -> false   (still RESOLVED)
clock = 09:00 + 72h      autoCloseIfStale() -> true    (CLOSED)
again                    autoCloseIfStale() -> false
```

## Edge cases

- Comments are kept in the order added, including those made in NEW, ON_HOLD and RESOLVED: `comment("one"); assign("ann"); comment("two"); hold("why")` → `["one", "two", "HOLD: why"]`.
- `assign(" ")`, `assign(null)`, `comment("")`, `hold(" ")`, `resolve("")` → `IllegalArgumentException`; the ticket is unchanged (still NEW, no agent, no comments).
- Modifying the list returned by `comments()` throws `UnsupportedOperationException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
