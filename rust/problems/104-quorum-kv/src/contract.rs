//! Thin test entry point. One instance is ONE replica of a fully replicated key-value store (Dynamo style,
//! leaderless, N = cluster size, W = R = majority). The tests run the nodes inside a deterministic simulation
//! (`playground_support::SimNet`): they deliver messages with `on_message`, advance time with `on_tick` (once per
//! simulated millisecond), drop / duplicate / delay messages and crash nodes (a crashed node keeps its memory but sees
//! nothing). Nodes may only communicate through the `send` function they were created with. Use interior mutability
//! (`Mutex`) for the state: the trait takes `&self`.
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
