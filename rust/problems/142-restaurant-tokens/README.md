# 142 · Restaurant Customer Tokens (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A busy restaurant hands every arriving customer a numbered **token** and calls the numbers over a handful of service counters. Some counters only handle take-away orders, some only seat dine-in parties, some do everything. VIP customers go first, take-away customers before dine-in guests, and otherwise it is first come first served; but nobody may be pushed back forever by a stream of higher-priority arrivals. Staff call the next customer, mark them served, mark them as a no-show when they do not turn up, and may let a no-show back in once. Customers (and a display board) want to know the position in line and a rough waiting time. You are building the token and queue core, driven by an injected clock.

## Problem

Build the token system for one restaurant day. Customers get a token `T-<n>` with a per-day sequence. Waiting tokens are served in one **serving order**: higher priority first (VIP, then TAKEAWAY, then DINE_IN), oldest first within a priority, with an anti-starvation rule that lifts a token that has been overtaken too often to the front. Counters ask for the next token they are able to serve. Called tokens must be served within a grace time or they become no-shows; a no-show can be recalled once within a recall window and then goes to the front of its priority class.

## Requirements

1. **Customer types**: `DINE_IN` (party size 1..12), `TAKEAWAY` and `VIP` (party size exactly 1). Priorities: VIP = 2, TAKEAWAY = 1, DINE_IN = 0.
2. `issueToken(customerName, type, partySize)` returns `"T-<n>"`, `n` = 1, 2, 3, ... in issue order for the day. A number is never reused until `resetDay()`. The new token is `WAITING`. A blank/null name, an unknown/null type, a DINE_IN party size outside 1..12 or a TAKEAWAY/VIP party size other than 1 is an error and consumes no number.
3. **Serving order** (what `queueSnapshot()` lists and `callNext` follows), over all `WAITING` tokens:
   1. first, every **promoted** token (rule 4), oldest token number first;
   2. then the other tokens by priority, highest first;
   3. within one priority, tokens that were recalled (rule 9) first, the most recently recalled in front, and then the remaining tokens by token number ascending.
4. **Anti-starvation**. Every token has a *pass-over count* (starts at 0). Whenever a token X is called, the pass-over count of every token that is `WAITING` at that moment **and has a smaller token number than X** goes up by one (it does not matter which counter called X, or whether the waiting token could have been served by that counter). A waiting token whose count is **5 or more** is *promoted*. A promoted token stays promoted while it waits. A cancelled token is no longer waiting and is not counted.
5. `addCounter(counterId, supportedTypes)` registers a counter that serves only those types (at least one; names must be valid; id non-blank and not yet registered; later changes to the caller's collection must not affect the counter).
6. `callNext(counterId)`: walks the serving order and takes the first waiting token whose type the counter supports. That token becomes `CALLED`, remembering the clock time of the call. Tokens the counter does not support are skipped and stay where they are (they are not consumed and keep their place). If nothing matches the result is empty. An unknown counter is an error. A counter may have any number of tokens `CALLED` at once.
7. `markServed(token)`: `CALLED` → `SERVED`, recording the service duration `now - calledAt`. `markNoShow(token)`: `CALLED` → `NO_SHOW`, remembering `now`. Any other state is an error. A `CALLED` token that is overdue but has not been expired yet can still be served.
8. `expireOverdue()` (invoked by tests; nothing happens by itself): every `CALLED` token with `now - calledAt > graceTimeMs` (strictly more) becomes `NO_SHOW`, stamped with the time of **this call**. Every `NO_SHOW` token with `now - noShowAt > recallWindowMs` (strictly more) becomes `EXPIRED`. The result lists the tokens that became `NO_SHOW` in this call, ordered by token number.
9. `recall(token)`: allowed only for a `NO_SHOW` token, at most once per token, and only while `now - noShowAt <= recallWindowMs` (the edge itself is still allowed). The token becomes `WAITING` again with its pass-over count reset to 0, placed at the front of its priority class (rule 3.3). A token that is `NO_SHOW` for the second time cannot be recalled again. A refused recall changes nothing (it does not use up the one recall).
10. `cancel(token)`: only from `WAITING`, to `CANCELLED`. Any other state is an error.
11. `status(token)` returns one of `WAITING`, `CALLED`, `SERVED`, `NO_SHOW`, `CANCELLED`, `EXPIRED`.
12. `positionInQueue(token)`: the 1-based index of the token in the serving order of all waiting tokens (not per counter). Error unless the token is `WAITING`.
13. `estimatedWaitMs(token)` = `(position - 1) * averageServiceMs`, where `averageServiceMs` is the **integer (floor) mean** of `servedTime - calledTime` over all tokens marked served so far today, or `300000` while none has been served. Only `markServed` contributes (once per token); no-shows and cancellations do not. Same errors as rule 12.
14. `queueSnapshot()` returns the waiting tokens in serving order (a copy).
15. `stats()` returns `"issued:served:noShow:cancelled:waiting"` where `issued` is the number of tokens issued today and the other four count tokens **currently** in that state; `noShow` counts `NO_SHOW` and `EXPIRED` together. `CALLED` tokens appear in none of the four.
16. `resetDay()` forgets every token and statistic (including the average service time), restarts numbering at `T-1` and keeps the registered counters. Afterwards an old token id is unknown.
17. Unknown token ids are errors in every operation that takes a token.
18. Thread safety: all operations may be called from many threads; 16 threads issuing tokens concurrently must get unique, gap-free numbers, and a token must never be called twice.

## Behavioural contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, TokenError>` with `InvalidArgument` (bad input, bad or unknown counter), `UnknownToken`, `InvalidState` (the token is known but in the wrong state, or a recall is not allowed). Types and statuses are plain `&str`/`String` values (constants in `contract.rs`). The clock is an injected `Arc<dyn playground_support::Clock>` (`now_millis()`); the grace time and recall window are passed to `create`.
- Java: `IllegalArgumentException` / `IllegalStateException` instead, with an injected `java.time.Clock`.

```rust
pub trait TokenSystem: Send + Sync {
    // names: DINE_IN, TAKEAWAY, VIP; WAITING, CALLED, SERVED, NO_SHOW, CANCELLED, EXPIRED; DEFAULT_SERVICE_MS = 300_000

    fn add_counter(&self, counter_id: &str, supported_types: &[&str]) -> Result<(), TokenError>;
    fn issue_token(&self, customer_name: &str, customer_type: &str, party_size: u32) -> Result<String, TokenError>; // "T-<n>"
    fn call_next(&self, counter_id: &str) -> Result<Option<String>, TokenError>;
    fn mark_served(&self, token: &str) -> Result<(), TokenError>;
    fn mark_no_show(&self, token: &str) -> Result<(), TokenError>;
    fn expire_overdue(&self) -> Vec<String>; // tokens that became NO_SHOW in this call, by token number
    fn recall(&self, token: &str) -> Result<(), TokenError>;
    fn cancel(&self, token: &str) -> Result<(), TokenError>;
    fn status(&self, token: &str) -> Result<String, TokenError>;
    fn position_in_queue(&self, token: &str) -> Result<usize, TokenError>; // 1-based
    fn estimated_wait_ms(&self, token: &str) -> Result<u64, TokenError>;
    fn queue_snapshot(&self) -> Vec<String>;
    fn stats(&self) -> String; // "issued:served:noShow:cancelled:waiting"
    fn reset_day(&self);
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(clock: Arc<dyn Clock>, grace_time_ms: u64, recall_window_ms: u64) -> Option<Arc<dyn TokenSystem>>
```

In the requirements the operations use their Java names (`issueToken`, `callNext`, ...); in Rust they are the snake_case methods above. The tests build a system with a `FakeClock` (grace 60000 ms, recall window 120000 ms) and register one counter `"all"` that supports all three types. Full doc comments are in `src/contract.rs`.

## What the tests check

These tests are black-box: they only call the contract above, so any type design behind it works. Time-dependent behaviour uses the injected fake clock, never real sleeping. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Numbering, invalid requests, party-size boundaries; serving order by priority then age; empty queue.
- Counters: skipping unsupported types without consuming, multi-type counters, invalid or unknown counters.
- State changes: served / no-show / cancel and their error cases; positions; wait estimates (default and integer mean).
- Anti-starvation: promotion on the fifth pass-over, only later-issued tokens count, counters that cannot serve the token still count, several promoted tokens oldest first.
- Grace expiry boundaries, serving an overdue token before expiry runs, recall to the front, one recall only, the recall window edge, `EXPIRED`.
- `stats()`, `reset_day()`, and concurrency: 16 threads issuing tokens, 8 counters calling concurrently.

## Examples

### Priorities and counters

```
addCounter("all", {DINE_IN, TAKEAWAY, VIP}); addCounter("tk", {TAKEAWAY})
issueToken("a", DINE_IN, 2) -> T-1      issueToken("b", TAKEAWAY, 1) -> T-2
issueToken("c", VIP, 1)     -> T-3      issueToken("d", DINE_IN, 3)  -> T-4
issueToken("e", VIP, 1)     -> T-5
queueSnapshot()                -> [T-3, T-5, T-2, T-1, T-4]
callNext("tk")                 -> T-2        (T-3 and T-5 are VIP: skipped, not consumed)
callNext("tk")                 -> empty
queueSnapshot()                -> [T-3, T-5, T-1, T-4]
```

### Anti-starvation, hand-verified

```
issueToken(.., DINE_IN, 2) -> T-1      then six VIP tokens T-2 .. T-7
callNext("all") -> T-2   T-1 waits, 1 < 2     count(T-1) = 1
callNext("all") -> T-3                         count = 2
callNext("all") -> T-4                         count = 3
callNext("all") -> T-5                         count = 4
queueSnapshot() -> [T-6, T-7, T-1]             (count 4: not promoted yet)
callNext("all") -> T-6                         count = 5 -> T-1 is promoted
queueSnapshot() -> [T-1, T-7]                  (promoted beats the VIP T-7)
callNext("all") -> T-1,  then T-7
```

A token issued *before* the waiting one never passes it over: with VIP tokens T-1..T-6 all served while DINE_IN T-7 waits, T-7 still has count 0, and a new VIP T-8 is served before it.

### Grace, no-show and recall (grace 60000 ms, recall window 120000 ms, clock at 0)

```
issue 4 DINE_IN tokens T-1..T-4; callNext -> T-1; callNext -> T-2        (both called at 0)
markNoShow(T-1); markNoShow(T-2)                                        (both no-show at 0)
issueToken(.., VIP, 1) -> T-5
queueSnapshot()  -> [T-5, T-3, T-4]
recall(T-1)      -> queueSnapshot() = [T-5, T-1, T-3, T-4]
recall(T-2)      -> queueSnapshot() = [T-5, T-2, T-1, T-3, T-4]   (the later recall is in front)
recall(T-1)      -> error                                          (already recalled once)

callNext -> T-5 at time 0; advance 60000; expireOverdue() -> []    (60000 is not "more than" the grace)
advance 1;          expireOverdue() -> [T-5]; status(T-5) = NO_SHOW (stamped at 60001)
advance 120001;     expireOverdue() -> [];    status(T-5) = EXPIRED; recall(T-5) -> error
```

### Estimates and statistics

```
issue 5 DINE_IN tokens; callNext -> T-1 at t; advance 1000; markServed(T-1)    (took 1000)
callNext -> T-2; advance 2001; markServed(T-2)                                (took 2001)
average = (1000 + 2001) / 2 = 1500 (integer division)
estimatedWaitMs(T-3) = 0      estimatedWaitMs(T-4) = 1500      estimatedWaitMs(T-5) = 3000
stats() = "5:2:0:0:3"
```

Before anything has been served, the same line gives `estimatedWaitMs(T-4) = 900000` (3 * 300000).

## Edge cases

- Failed `issueToken` calls consume no number; `T-<n>` follows `n = 1, 2, 3, ...` with no gaps even under concurrency.
- A counter that supports only some types may pass over many tokens; those tokens keep their place and their pass-over counts keep growing as later tokens are served by *any* counter.
- A token called by one counter is gone from the queue for every other counter.
- Promotion orders by token number only: a promoted DINE_IN `T-1` is served before a promoted TAKEAWAY `T-2`.
- A recalled token is never promoted on return (its count is reset); it later accumulates pass-overs again like any waiting token. Tokens served after the recall with a larger number than the recalled token count against it.
- Recalled tokens sit in front of their own class only: a VIP in the queue still goes before a recalled DINE_IN, and a promoted token still goes before everything.
- `expireOverdue` can turn one token `CALLED` → `NO_SHOW` and another `NO_SHOW` → `EXPIRED` in the same call; a token that has just become `NO_SHOW` is not expired by the same call.
- `status` of a `NO_SHOW` token stays `NO_SHOW` after the recall window until `expireOverdue()` runs, but `recall` is refused as soon as the window has passed.
- `positionInQueue`/`estimatedWaitMs` on a non-waiting token is an error (not 0).
- The average ignores tokens that were not served; it floors (`4 / 3 = 1`).
- `cancel` on a `CALLED` token is an error; so is `markServed` on a `WAITING` one.

## Glossary

- **Token**: the numbered ticket `T-<n>` a customer holds for the day.
- **Serving order**: the single order of all waiting tokens described in requirement 3.
- **Counter**: a service point that only takes the customer types it supports.
- **Pass-over count / promoted**: how many later-issued tokens were called while a token waited; promoted from 5 on.
- **Grace time**: how long a called token may take to be served before expiry turns it into a no-show.
- **Recall window**: how long after becoming a no-show a token may be recalled (once).

## Running

```bash
cargo test -p p142_restaurant_tokens                    # from rust/ (or `cargo test` from this directory)
STRICT=1 cargo test -p p142_restaurant_tokens           # fail instead of skip while unimplemented
cargo test -p p142_restaurant_tokens --features hidden  # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation print `SKIPPED` and return until `create` is wired up, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`, `run_concurrently`): `rust/support`.
