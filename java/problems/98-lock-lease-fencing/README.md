# 98 · Lease Lock with Fencing Tokens (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 50 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building a lock service for workers that must not run the same job concurrently. A lock is a lease: it is granted for a
number of milliseconds and expires unless renewed, so a crashed worker cannot hold it forever. Expiry alone is not enough: a
worker can freeze (a long garbage-collection pause) and wake up after its lease ran out, when another worker already
holds the lock, and still believe it is the owner. So every grant also returns a *fencing token*, a number that only ever
grows for that lock. A storage system can then refuse writes that carry an older token than one it has already seen, which makes a
stale owner harmless. Nothing in the service blocks or waits: a caller who cannot get the lock simply receives an empty answer.

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
9. `acquire` for a free or expired lock always issues a new token, even when the new owner is the same as the previous one. Only an unexpired lease held by the same owner is extended with the same token.
10. `renew` and `release` need ALL of: the right owner, the right token, and an unexpired lease; a failed call changes nothing. A successful `renew` keeps the token.
11. `latestToken(lock)` counts grants that issued a token; extending a lease, renewing, releasing and expiry never change it.
12. Lease times use the injected clock: a lease granted at time `t` for `d` ms is valid for `now` in `[t, t + d)`.

## Simulation rules and constraints

- Plain in-process object driven by an injected `FakeClock` (no network simulation); the tests call it from real threads, so it must be thread-safe.
- Nothing blocks and nothing sleeps: a lease is valid while `now < grantedAt + leaseMs`; at `now == grantedAt + leaseMs` it has expired. Time only moves when a test calls `clock.advance`.
- Under contention exactly one caller wins a free lock; mutual exclusion and strictly increasing tokens must hold with 16 threads taking the same lock 100 times each.
- The 'fenced store' in the tests, which rejects writes with a token lower than one it has seen, stands for the downstream system that makes a paused owner harmless; the service itself cannot stop a stale owner from running, only make it detectable.
- No faults are injected into the service; the faults are the stalls modelled by advancing the clock past a lease while an owner is 'paused'.

## Glossary

- **Lease**: permission to hold the lock for a limited time; expires on its own unless extended.
- **Fencing token**: a number issued with each new grant of a lock; strictly greater than any token issued before for the same lock. The first token is 1.
- **Stale owner**: an owner whose lease has expired (and possibly been taken over) but who has not noticed yet.
- **Fenced store**: a downstream system that remembers the highest token it has seen and rejects writes with a lower one. In the tests it is a simple class that does exactly this.

## Examples

**Example 1: takeover at expiry** (clock starts at 0)

```
acquire("job","ann",1000) @ t=0     -> Optional[1]     holder("job") = ann, latestToken("job") = 1
acquire("job","bob",1000) @ t=999   -> Optional.empty      ann's lease is still valid
holder("job") @ t=1000              -> empty               expired exactly at t + d
acquire("job","bob",1000) @ t=1000  -> Optional[2]     new, higher token
renew("job","ann",1,1000)           -> false                       stale owner
release("job","ann",1)              -> false                       holder is still bob
```

**Example 2: the owner re-acquires and renews**

```
acquire("job","ann",1000) @ t=0     -> token 1, valid until 1000
acquire("job","ann",1000) @ t=900   -> token 1 again; valid until 1900 (now + lease, not 1000 + lease)
holder("job") @ t=1800              -> ann
renew("job","bob",1,1000)           -> false (wrong owner);   renew("job","ann",2,1000) -> false (wrong token)
renew("job","ann",1,1000) @ t=1800  -> true; valid until 2800, still token 1
renew("job","ann",1,1000) @ t=2800  -> false (too late: expired exactly then)
acquire("job","ann",1000) @ t=2800  -> token 2 (her own lease had expired, so this is a new grant)
```

**Example 3: release**

```
acquire("job","ann",1000) -> 1;   release("job","bob",1) -> false;   release("job","ann",9) -> false
release("job","ann",1) -> true;   release("job","ann",1) -> false (already released)
acquire("job","bob",1000) -> 2    (tokens are not reused after a release)
```

**Example 4: why the token matters** (the fenced store from the tests rejects writes with a token lower than one it has seen)

```
ann acquires "db" (token 1), then stalls for 2 000 ms (lease 1 000 ms)
bob acquires "db" (token 2) and writes with token 2   -> accepted
ann wakes up and writes with token 1                   -> rejected; the stored value is still bob's
```
Contention: 16 threads that race for one free lock produce exactly one winner; 16 threads each taking and releasing one lock 100 times never overlap, and the 1 600 tokens they receive are strictly increasing (`latestToken` = 1 600).

## Edge cases

- A lease length of 0 or less is rejected (`IllegalArgumentException`) before anything changes.
- `holder` of a lock that was never taken, released, or expired is empty; `latestToken` of a never-taken lock is 0.
- Locks have independent state and independent token counters: `acquire("b", ..)` is token 1 even if `"a"` is at token 5.
- Release by the holder frees the lock immediately: another owner can acquire it at the same instant and gets `token + 1`.
- Eight threads racing for a free lock: exactly one wins; after the lease expires and eight race again, exactly one more wins (two winners, `latestToken` = 2).

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

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory. A lease granted at time t for d milliseconds is valid while
 * {@code now < t + d}; at {@code now == t + d} it has expired. Nothing blocks: a caller that does not get the lock
 * gets an empty answer and decides what to do.
 */
public interface LockService {

    /**
     * Tries to take the named lock for {@code leaseMs} (> 0, else IllegalArgumentException).
     * <ul>
     *   <li>Free, or the previous lease has expired: granted with a NEW fencing token = (highest token ever issued for
     *       this lock) + 1. Tokens only ever grow, also across different owners and expired leases.</li>
     *   <li>Held (unexpired) by the same owner: the lease is extended to now + leaseMs and the SAME token is returned.</li>
     *   <li>Held (unexpired) by somebody else: empty.</li>
     * </ul>
     */
    Optional<Long> acquire(String lock, String owner, long leaseMs);

    /** Extends the lease to now + leaseMs. True only if this owner and token still hold an unexpired lease. */
    boolean renew(String lock, String owner, long token, long leaseMs);

    /** Gives the lock up. True only if this owner and token still hold an unexpired lease; otherwise nothing changes. */
    boolean release(String lock, String owner, long token);

    /** The owner of the unexpired lease, if any. */
    Optional<String> holder(String lock);

    /** The highest token ever issued for the lock; 0 if it was never taken. */
    long latestToken(String lock);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static LockService create(Clock clock)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- First grant returns token 1 and sets holder/latestToken; another owner is refused while valid (at lease - 1 ms); locks are independent and an untouched lock has token 0.
- Takeover exactly at expiry with token 2; re-acquire by the same owner keeps its token and extends the lease; re-acquire after its own expiry gives a new token.
- Renew only with right owner/token before expiry; release only by the holder and not twice; a stale owner cannot release or renew the new holder's lock.
- A fenced store rejects the write of an owner whose lease expired while 'paused'.
- Non-positive leases rejected.
- Concurrency: 16 threads each acquire/release one lock 100 times - never two inside, 1,600 tokens strictly increasing, `latestToken` = 1,600; 16 threads race for a free lock - exactly one wins.
- **Repeat 100x**: eight threads race, the clock jumps past the lease, eight race again: exactly two winners, `latestToken` = 2.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Time and tokens are `long`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/98-lock-lease-fencing test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
