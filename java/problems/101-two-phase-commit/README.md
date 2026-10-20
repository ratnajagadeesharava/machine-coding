# 101 · Two-Phase Commit (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A payment spans three services, each with its own small key-value store. Either all three apply their part or none does. You are asked to build both sides of the classic two-phase commit protocol: the coordinator that drives a transaction and the participants (resource managers) that vote, lock, and apply.

The hard part is not the happy path. Messages are lost and duplicated, participants and the coordinator crash and are restarted from their durable logs, and the network partitions. Whatever happens, no two participants may ever disagree on the outcome of a transaction, a participant that voted YES must wait for the coordinator's decision, and nobody may hold locks forever once the system is healthy again.

## Requirements

1. The coordinator's `begin(txId, writesByParticipant)` starts a transaction: it logs a BEGIN record (participants and writes) and sends PREPARE to every participant involved. Invalid calls (empty map, unknown participant, duplicate transaction id) are rejected.
2. A participant votes NO if any key of the transaction is locked by another prepared transaction, or any value to write equals the text `REJECT`. Otherwise it locks the keys, appends a PREPARED record to its log BEFORE answering YES.
3. A NO vote decides ABORT at once; YES from every participant decides COMMIT; if nothing is decided 500 ms after `begin`, the coordinator decides ABORT. The decision is appended to the log BEFORE any message about it is sent (the commit point) and then sent to all participants of the transaction.
4. PREPAREs are re-sent every 100 ms to participants that have not voted; the decision is re-sent every 200 ms to participants that have not acknowledged it.
5. On DECISION a participant applies the writes (COMMIT) or discards them (ABORT), releases the locks, logs the decision and acknowledges it (duplicates are acknowledged again). `get` shows only committed values.
6. A participant keeps a prepared transaction in doubt (locks held) until the coordinator answers, asking with INQUIRE every 200 ms. The coordinator answers an INQUIRE with the decision, with ABORT if it has never heard of the transaction (presumed abort), and not at all while the vote is still running.
7. A node restarted as a new instance on its old log rebuilds its state: participants their store, locks and in-doubt set; the coordinator its transactions. A coordinator transaction with BEGIN but no decision is aborted immediately; every decided transaction is announced again.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. No locking is needed (the Rust traits take `&self`, so keep state in a `Mutex`).
- Time is simulated: one tick = 1 ms, `onTick(nowMs)` is called once per millisecond with the current time. Never read the wall clock, never sleep; derive every timeout from the `nowMs` you were handed.
- Network model: a message sent now is delivered after a latency of 1-5 ms by default (random per message, so messages can overtake each other). Tests can change the latency range, drop a fraction of the messages, duplicate a fraction, cut the network into partitions (messages across the cut are lost, including ones already in flight) and heal it.
- Crash model: a crashed node gets no ticks and no messages and its sends are ignored, but its memory is kept (it is paused, not wiped). After `recover` it simply continues with its first tick.
- Same seed, same run: any randomness of yours must come from the seed you are given, and iteration order of your collections must not influence behaviour (a replay is compared in the tests).
- A restart is modelled by creating a NEW instance on the SAME `DurableLog` and registering it in the simulation; everything not in the log is lost. Appends are atomic and durable when `append` returns.
- The tests inject message loss (20%), crashes and restarts of any node including the coordinator, partitions that isolate one node, and random combinations of those over 100 seeds.

## Examples

```
t=0    K.begin("t1", {A:{x:1}, B:{y:2}})   K logs BEGIN, sends PREPARE to A, B
t~3    A, B lock x / y, log PREPARED, vote YES
t~6    K has both YES -> logs COMMIT -> sends DECISION(COMMIT)   <- commit point
t~9    A, B apply writes, release locks, log decision, ACK
-- variant: K dies right after logging COMMIT, partitioned away:
A, B stay in doubt (locks held, x/y unreadable) ... until K restarts from its log, re-announces COMMIT.
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.coordinator(id, participants, send, log)` and `Solution.participant(id, coordinatorId, send, log)`; `begin` throws `IllegalArgumentException` for invalid input, `outcome` of an unknown id too. Rust: `create_coordinator` / `create_participant`; `begin` returns `Err(TxError::InvalidTransaction)`, `outcome` returns `Err(TxError::UnknownTransaction)`; `in_doubt()` is sorted.
- Timing constants: PREPARE re-send every 100 ms; vote timeout 500 ms after `begin` (ABORT); decision re-send every 200 ms until acknowledged; participant INQUIRE every 200 ms while in doubt.
- Write-ahead order: coordinator logs BEGIN before PREPARE and the decision before any DECISION message; participant logs PREPARED (with the writes) before it answers YES.
- PREPARE idempotence: already prepared -> YES again; already decided or refused -> consistent answer (NO for ABORT/refused, YES for COMMIT). A DECISION for a never-prepared transaction is remembered so that a late PREPARE is refused and cannot lock keys forever.
- Locks: `isLocked(key)` is true exactly while a prepared, undecided transaction holds the key. A conflicting PREPARE during that window gets NO. A second transaction on the key is fine once the first is decided.
- Recovery: acknowledgements are not persisted, so a restarted coordinator re-sends every decision until acknowledged again; an INQUIRE for a transaction the coordinator has no record of (e.g. empty log) is answered ABORT.
- `outcome` is PENDING until decided, then COMMITTED or ABORTED. The payload format and the log record format are yours.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Map;

/**
 * The transaction coordinator of two-phase commit. Runs inside the deterministic simulation ({@code testkit.SimNet}):
 * messages arrive through {@link #onMessage}, time through {@link #onTick} (once per simulated millisecond). Talks to the
 * participants only through the {@code send} function it was created with. Single-threaded.
 *
 * <p>Protocol:
 * <ol>
 *   <li>{@link #begin}: write a BEGIN record (with the participants and their writes) to the log, then send PREPARE to
 *       every participant involved. PREPAREs are re-sent every 100 ms to participants that have not voted yet.</li>
 *   <li>A NO vote decides ABORT at once. Votes of YES from all participants decide COMMIT. If no decision has been
 *       reached 500 ms after begin, ABORT is decided. The decision is appended to the log BEFORE any message about it
 *       is sent (that is the commit point), and then sent to every participant of the transaction.</li>
 *   <li>The decision is re-sent every 200 ms to participants that have not acknowledged it, until all have.</li>
 *   <li>A participant that asks about a transaction (INQUIRE) gets the decision if there is one, "ABORT" if the
 *       coordinator has never heard of the transaction (presumed abort), and no answer while the vote is still running.</li>
 *   <li>Restart: a new instance created with a non-empty log rebuilds its transactions from it. A transaction with a BEGIN
 *       but no decision is decided ABORT right away (presumed abort). Every decided transaction is announced to its
 *       participants again (acknowledgements are not persisted).</li>
 * </ol>
 * The payload format is yours.
 */
public interface CoordinatorNode {

    void onMessage(String from, String payload);

    void onTick(long nowMs);

    /**
     * Starts a transaction. {@code writesByParticipant} maps participant id to the key/value pairs it must write.
     * IllegalArgumentException for an empty map, an unknown participant or a duplicate transaction id.
     */
    void begin(String txId, Map<String, Map<String, String>> writesByParticipant);

    /** PENDING until decided. IllegalArgumentException for an unknown transaction. */
    Outcome outcome(String txId);
}

package contract;

import java.util.List;

/**
 * Stable storage of one node (provided by the tests; it survives a restart of the node: a restarted node is a NEW
 * instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
 */
public interface DurableLog {

    void append(String record);

    /** Everything appended so far, oldest first. */
    List<String> records();
}

package contract;

/** Provided. */
public enum Outcome {
    PENDING,
    COMMITTED,
    ABORTED
}

package contract;

import java.util.Optional;
import java.util.Set;

/**
 * A resource manager in two-phase commit: a small key-value store whose writes are applied atomically with the other
 * participants'. Runs inside the deterministic simulation; single-threaded.
 *
 * <p>Protocol:
 * <ul>
 *   <li>PREPARE(tx, writes): if the transaction was already prepared, answer YES again; if it was already decided or
 *       refused, answer consistently (NO for ABORT/refused, YES for COMMIT). Otherwise the participant votes NO if any
 *       key is locked by another prepared transaction, or any value to write equals the text {@code "REJECT"} (a
 *       stand-in for a business rule). On YES it locks the keys and appends a PREPARED record (with the writes) to its
 *       log BEFORE answering: after that it may not unilaterally change its mind.</li>
 *   <li>DECISION(tx, COMMIT|ABORT): COMMIT applies the writes; both release the locks; the decision is logged and
 *       acknowledged (acknowledge duplicates again). A decision for a transaction that was never prepared here is
 *       remembered too, so that a late PREPARE for it is refused instead of locking keys forever.</li>
 *   <li>In-doubt transactions (prepared, no decision) keep their locks and are asked about every 200 ms (INQUIRE), until
 *       the coordinator answers.</li>
 *   <li>Restart: a new instance created with a non-empty log rebuilds its store, locks and in-doubt set from it.</li>
 * </ul>
 */
public interface ParticipantNode {

    void onMessage(String from, String payload);

    void onTick(long nowMs);

    /** The last committed value of the key. */
    Optional<String> get(String key);

    /** True while a prepared (in-doubt) transaction holds the key. */
    boolean isLocked(String key);

    /** Transactions that are prepared and not decided yet. */
    Set<String> inDoubt();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
    public static CoordinatorNode coordinator(String id, List<String> participants, BiConsumer<String, String> send, DurableLog log)

    /** A participant that reports to {@code coordinatorId}. {@code log} may already contain records (restart). */
    public static ParticipantNode participant(String id, String coordinatorId, BiConsumer<String, String> send, DurableLog log)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Happy path: COMMIT with writes applied on involved participants only, locks released, nobody left in doubt.
- One participant votes NO (`REJECT`): ABORT everywhere, YES voters release their locks and nothing is written.
- Lock window: with 40 ms latency A holds the lock and has t1 in doubt; a conflicting t2 is ABORTED, t1 COMMITS; the value of t1 survives.
- A second transaction on the same key succeeds after the first finished.
- A silent (crashed) participant: PENDING before 500 ms, ABORTED after; the YES voter releases its lock. When the late participant returns it learns the ABORT and holds no lock.
- A YES voter that crashes stays in doubt through the COMMIT and applies it after recovery; the same after a restart from its log (locks and doubt rebuilt from the log).
- Blocking: the coordinator dies right after logging COMMIT behind a partition; participants stay in doubt with locks; after heal and restart on the same log the outcome is COMMITTED and everything completes.
- Presumed abort: coordinator restarts after BEGIN without a decision -> ABORTED; an inquiry about an unknown transaction does not break anything; invalid `begin` / `outcome` calls are rejected.
- 30 transactions on a 20%-loss network are all decided and atomic; 100 random crash/restart/partition schedules keep every transaction atomic, REJECT never commits, nobody remains in doubt.
- More edge cases (200 parallel transactions, a hot key contended by 50 transactions) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

The tests hand you a `DurableLog` of strings; pick a record format that is easy to parse back (e.g. a type tag and escaped fields) and be careful with separators inside values. Treat the instance as single-threaded.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/101-two-phase-commit test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
