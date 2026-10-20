//! Thin test entry point. One instance is ONE node of a cluster; the tests run the nodes inside a deterministic
//! simulation (`playground_support::SimNet`): they deliver messages with `on_message`, advance time with `on_tick`
//! (once per simulated millisecond) and read the state through the getters. Nodes may only communicate through the
//! `send` function they were created with. Use interior mutability (`Mutex`) for the state: the trait takes `&self`.
//!
//! The protocol is the election half of Raft:
//! * Every node starts as a FOLLOWER in term 0. A follower that has heard nothing from a leader (or granted a vote)
//!   for its election timeout becomes a CANDIDATE: term + 1, votes for itself, asks every peer for its vote. The
//!   election timeout is drawn uniformly from [150, 300] ms, again after every reset, from a random number
//!   generator seeded with the seed given to `create` (so a run is reproducible).
//! * A node grants at most one vote per term (to the first asker, or again to the same candidate) and resets its
//!   election timer when it does. A candidate with votes from a majority of the whole cluster (itself included)
//!   becomes LEADER and sends a heartbeat at once; a leader sends heartbeats to all peers every 50 ms.
//! * Any message carrying a higher term makes the receiver adopt that term and become FOLLOWER (forgetting its vote).
//!   Messages carrying a lower term are answered with the receiver's term and otherwise ignored. A candidate that
//!   receives a heartbeat of its own term becomes FOLLOWER. A heartbeat of the current term (or higher) resets the
//!   election timer and records the sender as the known leader.
//! * Check-quorum: a leader that has not got heartbeat replies from a majority of the cluster (itself included)
//!   within the last 300 ms steps down to FOLLOWER (same term) and starts a fresh election timer.
//! * A cluster of one node elects itself as soon as its election timer fires.
//!
//! The payload format is yours.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub trait ElectionNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    /// Called once per millisecond with the current simulated time (milliseconds since the start).
    fn on_tick(&self, now_ms: u64);

    fn role(&self) -> Role;

    fn term(&self) -> u64;

    /// The leader this node currently knows in its current term: itself if it is the leader, `None` if unknown.
    fn leader(&self) -> Option<String>;
}
