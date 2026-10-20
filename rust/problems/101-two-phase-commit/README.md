# 101 · Two-Phase Commit (Rust)

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

**In short:** you write two kinds of node: a coordinator (`begin`, `outcome`) and a participant (`get`, `is_locked`, `in_doubt`). They talk only through `send`, and each keeps a durable log that survives a restart. The tests check that every transaction ends COMMITTED on all participants or ABORTED on all, that locks are held and released at the right moments, and that restarts from the log do not change any decision.

## Requirements

1. The coordinator's `begin(txId, writesByParticipant)` starts a transaction: it logs a BEGIN record (participants and writes) and sends PREPARE to every participant involved. Invalid calls (empty map, unknown participant, duplicate transaction id) are rejected.
2. A participant votes NO if any key of the transaction is locked by another prepared transaction, or any value to write equals the text `REJECT`. Otherwise it locks the keys, appends a PREPARED record to its log BEFORE answering YES.
3. A NO vote decides ABORT at once; YES from every participant decides COMMIT; if nothing is decided 500 ms after `begin`, the coordinator decides ABORT. The decision is appended to the log BEFORE any message about it is sent (the commit point) and then sent to all participants of the transaction.
4. PREPAREs are re-sent every 100 ms to participants that have not voted; the decision is re-sent every 200 ms to participants that have not acknowledged it.
5. On DECISION a participant applies the writes (COMMIT) or discards them (ABORT), releases the locks, logs the decision and acknowledges it (duplicates are acknowledged again). `get` shows only committed values.
6. A participant keeps a prepared transaction in doubt (locks held) until the coordinator answers, asking with INQUIRE every 200 ms. The coordinator answers an INQUIRE with the decision, with ABORT if it has never heard of the transaction (presumed abort), and not at all while the vote is still running.
7. A node restarted as a new instance on its old log rebuilds its state: participants their store, locks and in-doubt set; the coordinator its transactions. A coordinator transaction with BEGIN but no decision is aborted immediately; every decided transaction is announced again.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. The Rust traits take `&self`.
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

A NO vote ends the transaction at once:

```
K.begin("t2", {A:{x:1}, B:{y:"REJECT"}, C:{z:3}})
B refuses (a value equals "REJECT") and votes NO
K decides ABORT immediately, without waiting for C's vote, logs it, and sends DECISION(ABORT) to A, B and C
A and C release the locks they may have taken and write nothing; x, y and z stay unwritten everywhere
```

Lock conflict and release:

```
t1 is prepared on A and holds key "k"  -> A.is_locked("k") == true, A.in_doubt() == {"t1"}, A.get("k") still shows the OLD value
K.begin("t2", {A:{k:"other"}})          -> A votes NO (k is locked by t1) -> t2 ABORTED; t1 is not affected
t1 is decided and applied                   -> A.is_locked("k") == false, A.get("k") == t1's value
K.begin("t3", {A:{k:"later"}})          -> now A votes YES; t3 can commit
```

Timeout:

```
B is down. K.begin("t1", {A:{x:1}, B:{y:2}})
PREPARE goes to A (YES) and is re-sent to B every 100 ms, but B never answers
outcome("t1") stays PENDING until 500 ms after begin, then K decides ABORT and keeps re-sending DECISION(ABORT) every 200 ms
A releases its lock when it hears the decision; B learns the ABORT whenever it comes back
```

Restart and presumed abort:

```
K crashes after logging BEGIN but before any decision; a NEW coordinator is created on the same log
-> outcome("t1") == ABORTED right away, DECISION(ABORT) is sent to the participants of t1
A coordinator started on an EMPTY log that receives INQUIRE about "t1" answers ABORT (it has never heard of t1);
outcome("t1") on that coordinator fails with `Err(TxError::UnknownTransaction)`
```

Late PREPARE:

```
A receives DECISION(ABORT) for "t9" although it never saw a PREPARE for t9, and remembers it
a delayed PREPARE for "t9" then arrives -> A answers NO and takes no locks (otherwise they would be held forever)
```

## Edge cases

- Invalid `begin` (`Err(TxError::InvalidTransaction)`): an empty map, a participant that is not one of the coordinator's participants, or a `tx_id` that was used before. `outcome` of an id that was never begun fails with `Err(TxError::UnknownTransaction)`.
- Only the participants named in `writes` take part in a transaction (in the examples above, C is not involved in `t1`: it receives nothing and `C.get("x")` stays empty).
- A duplicate or re-sent PREPARE for a transaction that is already prepared on this participant gets YES again; the transaction's own lock does not make it vote NO against itself.
- A duplicate DECISION is acknowledged again but applied once. A DECISION is applied atomically: all keys of the transaction on that participant appear together.
- "REJECT" is checked against the VALUES to write, not against keys.
- A prepared transaction's writes are invisible to `get` until the COMMIT decision is applied; an ABORT leaves the old values untouched.
- The coordinator does not answer an INQUIRE while the vote of that transaction is still running; the participant keeps asking every 200 ms.
- Acknowledgements are not stored in the log, so after a restart a coordinator re-sends every decision until it has fresh acknowledgements.
- `in_doubt()` lists only transactions that are prepared and not decided; after the decision it is empty again.

## Glossary

- **Coordinator**: the node that drives a transaction and makes the final COMMIT / ABORT decision.
- **Participant (resource manager)**: a node with its own key-value store that votes on and applies its share of a transaction.
- **PREPARE / vote**: the first phase. A participant that votes YES promises it can commit if told to and may not change its mind.
- **Commit point**: the moment the coordinator has appended the decision to its log; from then on the outcome is fixed even if the coordinator crashes.
- **In doubt**: a participant has voted YES but has not yet heard the decision; it must keep its locks and wait.
- **Presumed abort**: when the coordinator has no record of a transaction, the answer is ABORT.
- **Durable log**: stable storage that survives a restart; a restarted node is a new object created on the same log, and anything not in the log is gone.
- **Write-ahead**: the log record is appended before the message that depends on it is sent.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.coordinator(id, participants, send, log)` and `Solution.participant(id, coordinatorId, send, log)`; `begin` throws `IllegalArgumentException` for invalid input, `outcome` of an unknown id too. Rust: `create_coordinator` / `create_participant`; `begin` returns `Err(TxError::InvalidTransaction)`, `outcome` returns `Err(TxError::UnknownTransaction)`; `in_doubt()` is sorted.
- Timing constants: PREPARE re-send every 100 ms; vote timeout 500 ms after `begin` (ABORT); decision re-send every 200 ms until acknowledged; participant INQUIRE every 200 ms while in doubt.
- Write-ahead order: coordinator logs BEGIN before PREPARE and the decision before any DECISION message; participant logs PREPARED (with the writes) before it answers YES.
- PREPARE idempotence: already prepared -> YES again; already decided or refused -> consistent answer (NO for ABORT/refused, YES for COMMIT). A DECISION for a never-prepared transaction is remembered so that a late PREPARE is refused and cannot lock keys forever.
- Locks: `isLocked(key)` is true exactly while a prepared, undecided transaction holds the key. A conflicting PREPARE during that window gets NO. A second transaction on the key is fine once the first is decided.
- Recovery: acknowledgements are not persisted, so a restarted coordinator re-sends every decision until acknowledged again; an INQUIRE for a transaction the coordinator has no record of (e.g. empty log) is answered ABORT.
- `outcome` is PENDING until decided, then COMMITTED or ABORTED. The payload format and the log record format are yours.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point for two-phase commit. Both node types run inside the deterministic simulation
//! (`playground_support::SimNet`): messages arrive through `on_message`, time through `on_tick` (once per simulated
//! millisecond). Nodes talk to each other only through the `send` function they were created with. The traits
//! take `&self`.

use std::sync::Arc;

/// Stable storage of one node (provided by the tests; it survives a restart of the node: a restarted node is a NEW
/// instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
pub trait DurableLog: Send + Sync {
    fn append(&self, record: &str);

    /// Everything appended so far, oldest first.
    fn records(&self) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Committed,
    Aborted,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TxError {
    /// empty writes, unknown participant, duplicate transaction id
    InvalidTransaction,
    UnknownTransaction,
}

/// Writes of one participant: `(participant id, [(key, value), ...])`.
pub type ParticipantWrites<'a> = (&'a str, &'a [(&'a str, &'a str)]);

/// The transaction coordinator.
///
/// Protocol:
/// 1. `begin`: write a BEGIN record (with the participants and their writes) to the log, then send PREPARE to every
///    participant involved. PREPAREs are re-sent every 100 ms to participants that have not voted yet.
/// 2. A NO vote decides ABORT at once. Votes of YES from all participants decide COMMIT. If no decision has been
///    reached 500 ms after begin, ABORT is decided. The decision is appended to the log BEFORE any message about it
///    is sent (that is the commit point), and then sent to every participant of the transaction.
/// 3. The decision is re-sent every 200 ms to participants that have not acknowledged it, until all have.
/// 4. A participant that asks about a transaction (INQUIRE) gets the decision if there is one, "ABORT" if the
///    coordinator has never heard of the transaction (presumed abort), and no answer while the vote is still running.
/// 5. Restart: a new instance created with a non-empty log rebuilds its transactions from it. A transaction with a
///    BEGIN but no decision is decided ABORT right away (presumed abort). Every decided transaction is announced to
///    its participants again (acknowledgements are not persisted).
///
/// The payload format is yours.
pub trait CoordinatorNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    fn on_tick(&self, now_ms: u64);

    /// Starts a transaction.
    fn begin(&self, tx_id: &str, writes: &[ParticipantWrites<'_>]) -> Result<(), TxError>;

    /// `Pending` until decided.
    fn outcome(&self, tx_id: &str) -> Result<Outcome, TxError>;
}

/// A resource manager in two-phase commit: a small key-value store whose writes are applied atomically with the
/// other participants'.
///
/// Protocol:
/// * PREPARE(tx, writes): if the transaction was already prepared, answer YES again; if it was already decided or
///   refused, answer consistently (NO for ABORT/refused, YES for COMMIT). Otherwise the participant votes NO if any
///   key is locked by another prepared transaction, or any value to write equals the text `"REJECT"` (a stand-in for
///   a business rule). On YES it locks the keys and appends a PREPARED record (with the writes) to its log BEFORE
///   answering: after that it may not unilaterally change its mind.
/// * DECISION(tx, COMMIT|ABORT): COMMIT applies the writes; both release the locks; the decision is logged and
///   acknowledged (acknowledge duplicates again). A decision for a transaction that was never prepared here is
///   remembered too, so that a late PREPARE for it is refused instead of locking keys forever.
/// * In-doubt transactions (prepared, no decision) keep their locks and are asked about every 200 ms (INQUIRE), until
///   the coordinator answers.
/// * Restart: a new instance created with a non-empty log rebuilds its store, locks and in-doubt set from it.
pub trait ParticipantNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    fn on_tick(&self, now_ms: u64);

    /// The last committed value of the key.
    fn get(&self, key: &str) -> Option<String>;

    /// True while a prepared (in-doubt) transaction holds the key.
    fn is_locked(&self, key: &str) -> bool;

    /// Transactions that are prepared and not decided yet, sorted.
    fn in_doubt(&self) -> Vec<String>;
}

pub type Log = Arc<dyn DurableLog>;
```

Your wiring point (`src/lib.rs`):

```rust
/// A coordinator for the given participants. `log` may already contain records (restart).
pub fn create_coordinator(_id: &str, _participants: &[&str], _send: SendFn, _log: Log) -> Option<Arc<dyn CoordinatorNode>> { ... }

/// A participant that reports to `coordinator_id`. `log` may already contain records (restart).
pub fn create_participant(_id: &str, _coordinator_id: &str, _send: SendFn, _log: Log) -> Option<Arc<dyn ParticipantNode>> { ... }
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

The traits take `&self`. `in_doubt` output is in a deterministic order.

## Running

```bash
cargo test -p p101_two_phase_commit
STRICT=1 cargo test -p p101_two_phase_commit   # fail instead of skip while unimplemented
cargo test -p p101_two_phase_commit --features hidden   # interviewer scenarios
cargo play run 101              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.
