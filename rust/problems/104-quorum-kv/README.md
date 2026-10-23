# 104 · Quorum Key-Value Store (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

We are building a small replicated key-value store in the style of Dynamo/Cassandra. Every node holds a full copy of the data and any node can accept a client request and act as its coordinator; there is no leader. A write is successful when a majority has stored it, and a read consults a majority, so a read always overlaps with the latest acknowledged write.

Build one node. Clients are asynchronous: they start a request with an id and poll for the reply. The network loses, duplicates and delays messages, and nodes crash and recover (keeping their memory). Your job is to make acknowledged writes visible to every later read, to repair replicas that fall behind, and to fail cleanly (not hang, not lie) when no majority is reachable.

**In short:** one replica of a leaderless key-value store. A client calls `put` / `get` on any node with a fresh request id and later polls `reply(request_id)`. You make a put succeed once a majority has stored it, make a get consult a majority and return the newest value it sees, repair stale replicas, and give up with FAILED about a second after a request began if no majority answers.

## Requirements

1. `put(requestId, key, value)` and `get(requestId, key)` start a request on this node; `reply(requestId)` polls the result: PENDING, OK (with the value for a get), NOT_FOUND, or FAILED. A reply for an unknown id is PENDING.
2. A put gets the version (t, coordinatorId), where t is the last time passed to `onTick`. The higher version wins; equal t is decided by the larger id. A replica overwrites an entry only with a higher version, but acknowledges every write it receives (also an old one).
3. Put: the coordinator writes locally (one ack) and sends the write to all other nodes. The reply is OK as soon as a majority has acknowledged. Nodes that did not acknowledge are re-sent the write every 100 ms until all did or 1000 ms after the request started - also after the reply is already OK. No majority within 1000 ms: FAILED.
4. Get: the coordinator reads locally (one answer) and asks all others, re-asking silent ones every 100 ms. When a majority has answered, the reply is the entry with the highest version among the answers (OK), or NOT_FOUND if none has the key. No majority within 1000 ms: FAILED.
5. Read repair: every node that answered a get with an older entry (or none) is sent the winning entry as a write - also when its answer arrives after the reply was decided (until 1000 ms).
6. A request id is used once: starting a second request with a known id does nothing (client retries are safe) and does not change the stored reply.
7. `local(key)` exposes this replica's own copy (no coordination) so the tests can inspect replicas.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. The Rust traits take `&self`.
- Time is simulated: one tick = 1 ms, `onTick(nowMs)` is called once per millisecond with the current time. Never read the wall clock, never sleep; derive every timeout from the `nowMs` you were handed.
- Network model: a message sent now is delivered after a latency of 1-5 ms by default (random per message, so messages can overtake each other). Tests can change the latency range, drop a fraction of the messages, duplicate a fraction, cut the network into partitions (messages across the cut are lost, including ones already in flight) and heal it.
- Crash model: a crashed node gets no ticks and no messages and its sends are ignored, but its memory is kept (it is paused, not wiped). After `recover` it simply continues with its first tick.
- Same seed, same run: any randomness of yours must come from the seed you are given, and iteration order of your collections must not influence behaviour (a replay is compared in the tests).
- A node that was crashed learns the time again with its first tick after recovery; versions use the last time passed to `onTick`, so two puts in the same millisecond on different coordinators get the same t and are ordered by node id.
- The tests inject: crashed replicas (including 2 of 5), a minority/majority partition, 40% message loss, 50% duplication, and random sequences of crashes, loss and partitions over many seeds, with up to 5 nodes.

## Examples

```
cluster n1 n2 n3 (W = R = 2)
t=0    n1.put(r1,"k","v1")  -> n1 stores (0,n1) + sends WRITE to n2, n3
t~4    n2 acks -> majority (n1+n2) -> reply(r1) = OK ;   n3 crashed: re-sent every 100 ms until t=1000
t=900  n3 recovers, receives the re-sent write, acks
t=950  n3.get(r2,"k") : n3 answers itself + n1/n2 -> highest version (0,n1) -> OK "v1"
```

In Rust a successful put is `Reply::PutOk` and a successful get is `Reply::Value(v)`.

No majority reachable (n2 and n3 crashed, request started on n1 at t=0):

```
n1.put(r1,"k","v") and n1.get(r2,"k")
t=500   both replies are still Pending   (one answer out of 3 is not a majority; a request must not fail early)
t~1000  both replies become Failed      (observed by t=1100); the failed put may still sit on n1's own copy
```

Same millisecond, two coordinators:

```
t=0 (same tick)  n1.put(r1,"k","from-n1")   and   n3.put(r2,"k","from-n3")
versions (0,n1) and (0,n3): equal time, the larger id wins -> every replica ends with "from-n3"
```

Read repair:

```
n3 was down while n1.put(...,"k","v1") succeeded on n1+n2; n3 is back and n3.local("k") is empty
n1.get(r9,"k"): n1 and n2 answer "v1" with version (t,n1) -> majority -> Value("v1")
n3's answer says "I have nothing" (older) -> n1 sends the winning entry to n3 as a write
shortly after, n3.local("k") == "v1"   (this also happens if n3's answer arrives after the reply was already decided)
```

Partition `{n1} | {n2, n3}`:

```
n1.put(...)  -> Failed after ~1000 ms (only 1 of 3 acks)        n2.put(...) -> OK (n2 + n3 = 2 of 3)
after heal: n1.get(...) returns the majority's value (n1 asks n2, n3 and gets the newest version)
```

Quorum sizes: 3 nodes need 2 answers, 5 nodes need 3, always counting the coordinator itself as one.

## Edge cases

- `get` of a key that no answering replica holds is `Reply::NotFound`, but only once a majority has answered; before that it is `Reply::Pending`.
- `reply` of a request id that was never started is `Reply::Pending`. Starting a request with an id that is already known does nothing, even with a different key or value, and the stored reply stays as it was.
- A put keeps being re-sent to non-acknowledging nodes until all have acknowledged or 1000 ms have passed since the request began, even after the reply is already OK.
- A replica acknowledges every write it receives, also one with an older version than it holds, but keeps the entry with the higher version (a stale re-send must never overwrite newer data).
- An acknowledgement or answer from the same node counted twice (duplicate message) must not fake a majority.
- The version time `t` is the last time passed to `on_tick` on the coordinating node. A node that was crashed and recovered learns the time again with its first tick.
- The guarantee the tests check: a get that starts after an OK put returns that value or a later one, from any live coordinator, even if the replicas that were up during the write are not the ones that are up during the read.
- `local(key)` only looks at this node's own copy and never sends anything.

## Glossary

- **Coordinator**: the node a client happened to contact; it drives that one request.
- **Replica**: a node holding a full copy of the data.
- **Quorum / majority**: more than half of the cluster; any two majorities overlap in at least one node, which is why a read sees the last acknowledged write.
- **Version**: (time of the coordinator's last tick, coordinator id); higher time wins, then the larger id.
- **Ack**: a replica's confirmation that it received a write.
- **Read repair**: after a get, sending the winning entry to replicas that answered with an older entry (or none).
- **Request id**: the client's handle for an asynchronous request; used once.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.node(id, cluster, send)` with `cluster` including this node; `Reply(status, value)` with `Status.{PENDING, OK, NOT_FOUND, FAILED}` and `value` only for a successful get. Rust: `create(id, cluster, send)`; `Reply::{Pending, PutOk, Value(String), NotFound, Failed}`.
- Quorum = majority of the cluster size (2 of 3, 3 of 5); the coordinator itself counts as one answer / ack.
- Version = (t, coordinatorId) with t = last `onTick` time; compare t first, then the id as a string. Acknowledge every received write, even a stale one, but only overwrite with a higher version.
- Timing: re-send/re-ask every 100 ms to silent nodes; the request is abandoned (FAILED if no majority yet) 1000 ms after it started; a put keeps re-sending to unacknowledged nodes until 1000 ms even after OK; read repair continues until 1000 ms.
- PENDING until a majority has answered (a request must stay PENDING before the deadline, not fail early) and FAILED within about 1100 ms when no majority is reachable. A failed put may still have reached some replicas.
- Idempotence: the second `put`/`get` with a used request id is ignored; replicas must also tolerate duplicated WRITE / READ messages and duplicate acks (an ack counted twice must not fake a majority).
- Guarantee the tests check: a read after an OK put (from any live coordinator) returns that value or a later one; a read of a never-written key is NOT_FOUND. The payload format is yours.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. One instance is ONE replica of a fully replicated key-value store (Dynamo style,
//! leaderless, N = cluster size, W = R = majority). The tests run the nodes inside a deterministic simulation
//! (`playground_support::SimNet`): they deliver messages with `on_message`, advance time with `on_tick` (once per
//! simulated millisecond), drop / duplicate / delay messages and crash nodes (a crashed node keeps its memory but sees
//! nothing). Nodes may only communicate through the `send` function they were created with. The trait takes
//! `&self`.
//!
//! A node that was crashed learns the time again with its first tick after the recovery.
//!
//! Any node can be the coordinator of a request; clients are asynchronous: they start a request with an id and poll
//! `reply`. Rules:
//! * **Versions.** A put gets the version `(t, coordinator_id)` where `t` is the last time passed to `on_tick`. The
//!   higher version wins, equal `t` is decided by the larger id. A replica only overwrites an entry with a higher
//!   version and acknowledges every write it receives (also an old one).
//! * **put.** The coordinator writes locally (that is one ack) and sends the write to all others. The reply is `Ok`
//!   as soon as a majority acknowledged. Unacknowledged nodes are re-sent the write every 100 ms until all of them
//!   acknowledged or 1000 ms after the request started, also after the reply was already `Ok`. No majority within
//!   1000 ms: `Failed` (the write may still have reached some replicas).
//! * **get.** The coordinator reads locally (one answer) and asks all others, re-asking silent ones every 100 ms.
//!   When a majority answered, the reply is the entry with the highest version among the answers (`Ok`), or
//!   `NotFound` if none of them has the key. **Read repair:** every node that answered with an older entry (or none)
//!   gets the winning entry sent as a write, also if its answer arrives after the reply was decided (until 1000 ms).
//!   No majority within 1000 ms: `Failed`.
//! * A request id is used once; starting a second request with a known id does nothing (client retries are safe).
//!   `reply` of an unknown id is `Pending`.
//!
//! The payload format is yours.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// not finished yet
    Pending,
    /// put: stored on a majority
    PutOk,
    /// get: a majority answered and a value exists
    Value(String),
    /// get: a majority answered and none of them has the key
    NotFound,
    /// a majority did not answer within 1000 ms of the request
    Failed,
}

pub trait KvNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    /// Called once per millisecond with the current simulated time (milliseconds since the start).
    fn on_tick(&self, now_ms: u64);

    fn put(&self, request_id: &str, key: &str, value: &str);

    fn get(&self, request_id: &str, key: &str);

    fn reply(&self, request_id: &str) -> Reply;

    /// This replica's own copy, for inspection (no coordination).
    fn local(&self, key: &str) -> Option<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
/// * `id`: this node's name
/// * `cluster`: every node's name INCLUDING this one
/// * `send`: `send(to, payload)` delivers a message to another node (later, maybe never)
pub fn create(_id: &str, _cluster: &[&str], _send: SendFn) -> Option<Arc<dyn KvNode>> { ... }
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- A write on one node is readable from all three; an unknown key is NOT_FOUND.
- With two of three nodes crashed, put stays PENDING for 500 ms and both put and get are FAILED within 1.1 s.
- One node down: put and get still succeed. A write acknowledged while n3 was down is read correctly from n3 after n3 recovered and n1 crashed (quorum overlap).
- Read repair: a stale or empty replica receives the winning value after a read, also when its answer comes in after the reply was decided.
- 40% message loss: the put still succeeds and the re-sends bring both other replicas up to date within the deadline.
- Partition 1 | 2: the lone node's put is FAILED, the majority side's put is OK; after healing a read through the old minority returns the majority's value.
- Newer writes win; two puts in the same millisecond on different coordinators converge on the larger node id on every replica.
- A reused request id is ignored; a reply for an unknown id is PENDING; duplicated messages are harmless.
- 100 random schedules (crashes, 10% loss, mixed puts and gets): every get after an OK put returns the last acknowledged value; a never-written key reads NOT_FOUND.
- More edge cases (5-node majority of three, rotating crashes and partitions, convergence after healing) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Versions order by `(u64, String)`. Iteration order over pending requests must be deterministic.

## Running

```bash
cargo test -p p104_quorum_kv
STRICT=1 cargo test -p p104_quorum_kv   # fail instead of skip while unimplemented
cargo test -p p104_quorum_kv --features hidden   # interviewer scenarios
cargo play run 104              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.
