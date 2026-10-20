# 98 · Lease Lock with Fencing Tokens (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 50 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Several workers must not run the same job at the same time, so they ask a lock service for a named lock. A worker that crashes
must not hold the lock forever, so a lock is a lease: it expires unless renewed. But expiry alone is not safe: a worker can stall
(a long garbage-collection pause, a stopped VM) and wake up after its lease ran out, when somebody else already holds the lock - and it
will still believe it is the owner. Build the lock service so that storage systems can reject such a stale owner. Nothing in
the service blocks; callers that do not get the lock simply get an empty answer.

## Requirements

1. `acquire(lock, owner, leaseMs)` tries to take the named lock for `leaseMs` (> 0, else `IllegalArgumentException` / `Err(InvalidLease)`).
2. If the lock is free or its previous lease has expired, it is granted with a **new fencing token**: the highest token ever issued for this lock plus one (the first token is 1). Tokens grow across owners and across expired leases.
3. If the lock is held (unexpired) by the same owner, the lease is extended to `now + leaseMs` and the same token is returned (no new token).
4. If the lock is held (unexpired) by another owner, the answer is empty (`Optional.empty()` / `None`).
5. `renew(lock, owner, token, leaseMs)` extends the lease to `now + leaseMs` and returns true only if that owner and that token still hold an unexpired lease.
6. `release(lock, owner, token)` frees the lock and returns true only if that owner and token still hold an unexpired lease; otherwise nothing changes. A later `acquire` gets `token + 1`.
7. `holder(lock)` is the owner of the unexpired lease, if any. `latestToken(lock)` is the highest token ever issued (0 if never taken).
8. Locks are independent of one another.

## Simulation rules and constraints

- Plain in-process object driven by an injected `FakeClock` (no network simulation); the tests call it from real threads, so it must be thread-safe.
- Nothing blocks and nothing sleeps: a lease is valid while `now < grantedAt + leaseMs`; at `now == grantedAt + leaseMs` it has expired. Time only moves when a test calls `clock.advance`.
- Under contention exactly one caller wins a free lock; mutual exclusion and strictly increasing tokens must hold with 16 threads taking the same lock 100 times each.
- The 'fenced store' in the tests, which rejects writes with a token lower than one it has seen, stands for the downstream system that makes a paused owner harmless; the service itself cannot stop a stale owner from running, only make it detectable.
- No faults are injected into the service; the faults are the stalls modelled by advancing the clock past a lease while an owner is 'paused'.

## Examples

```
acquire("job","ann",1000) @t=0     -> Optional[1]
acquire("job","bob",1000) @t=999   -> empty            (ann's lease is valid)
holder("job") @t=1000              -> empty            (expired exactly at t+d)
acquire("job","bob",1000) @t=1000  -> Optional[2]      (new, higher token)
renew("job","ann",1, 1000)         -> false            (stale owner)

fenced store: bob writes with token 2 -> accepted; ann wakes up and writes with token 1 -> rejected
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `LockService` with `Optional<Long> acquire`, `boolean renew/release`, `Optional<String> holder`, `long latestToken`; factory `Solution.create(Clock)`. Rust: `acquire -> Result<Option<u64>, LockError>` (`InvalidLease` for 0 ms), `holder -> Option<String>`, `u64` tokens and `u64` lease, `create(Arc<dyn Clock>)`.
- Expiry is exclusive at the boundary: valid while `now < t + d`. Ann acquires at t=0 for 1,000 ms: at 999 Bob is refused, at 1,000 `holder` is empty and Bob gets token 2.
- Re-acquire by the current owner before expiry keeps the token and sets the new expiry to `now + leaseMs` (not `old expiry + leaseMs`); acquiring again after the owner's own lease expired issues a **new** token.
- Renew needs the right owner and the right token and an unexpired lease: wrong owner -> false, wrong token -> false, one millisecond too late -> false. A successful renew does not change the token.
- Release by the holder with the right token returns true once; a second release returns false and the next acquirer gets `token + 1` (tokens are not reused after release).
- A stale owner (old token, lease expired, someone else holds it) can neither renew nor release; `holder` still reports the new owner.
- `latestToken` is the counter of issued tokens: unaffected by release or expiry, 0 for a lock never taken.
- Non-positive lease lengths are rejected before any state changes.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`. A lease granted at time t for d milliseconds is valid while
//! `now < t + d`; at `now == t + d` it has expired. Nothing blocks: a caller that does not get the lock gets `None`
//! and decides what to do.

#[derive(Debug, PartialEq, Eq)]
pub enum LockError {
    /// lease of 0 milliseconds
    InvalidLease,
}

pub trait LockService: Send + Sync {
    /// Tries to take the named lock for `lease_ms` (> 0).
    /// * Free, or the previous lease has expired: granted with a NEW fencing token = (highest token ever issued for
    ///   this lock) + 1. Tokens only ever grow, also across different owners and expired leases.
    /// * Held (unexpired) by the same owner: the lease is extended to now + lease_ms and the SAME token is returned.
    /// * Held (unexpired) by somebody else: `Ok(None)`.
    fn acquire(&self, lock: &str, owner: &str, lease_ms: u64) -> Result<Option<u64>, LockError>;

    /// Extends the lease to now + lease_ms. True only if this owner and token still hold an unexpired lease.
    fn renew(&self, lock: &str, owner: &str, token: u64, lease_ms: u64) -> bool;

    /// Gives the lock up. True only if this owner and token still hold an unexpired lease; otherwise nothing changes.
    fn release(&self, lock: &str, owner: &str, token: u64) -> bool;

    /// The owner of the unexpired lease, if any.
    fn holder(&self, lock: &str) -> Option<String>;

    /// The highest token ever issued for the lock; 0 if it was never taken.
    fn latest_token(&self, lock: &str) -> u64;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn LockService>> { ... }
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- First grant returns token 1 and sets holder/latestToken; another owner is refused while valid (at lease - 1 ms); locks are independent and an untouched lock has token 0.
- Takeover exactly at expiry with token 2; re-acquire by the same owner keeps its token and extends the lease; re-acquire after its own expiry gives a new token.
- Renew only with right owner/token before expiry; release only by the holder and not twice; a stale owner cannot release or renew the new holder's lock.
- A fenced store rejects the write of an owner whose lease expired while 'paused'.
- Non-positive leases rejected.
- Concurrency: 16 threads each acquire/release one lock 100 times - never two inside, 1,600 tokens strictly increasing, `latestToken` = 1,600; 16 threads race for a free lock - exactly one wins.
- **Repeat 100x**: eight threads race, the clock jumps past the lease, eight race again: exactly two winners, `latestToken` = 2.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

A `Mutex<HashMap<String, LockState>>` is enough for the tests; if you want less contention shard it or use `DashMap`-style per-entry locking by hand. Keep time arithmetic in `u64` and use `saturating_add` for `now + lease`.

## Running

```bash
cargo test -p p98_lock_lease_fencing
STRICT=1 cargo test -p p98_lock_lease_fencing   # fail instead of skip while unimplemented
cargo test -p p98_lock_lease_fencing --features hidden   # interviewer scenarios
cargo play run 98              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.
