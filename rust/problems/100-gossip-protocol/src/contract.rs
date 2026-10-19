//! Thin test entry point. One instance is ONE node; the tests run the nodes inside a deterministic simulation
//! (`playground_support::SimNet`): they deliver messages with `on_message`, advance time with `on_tick` (once per
//! simulated millisecond) and read state through the getters. Nodes may only communicate through the `send` function
//! they were created with. Use interior mutability (`Mutex`) for the state: the trait takes `&self`.
//!
//! Protocol (push-pull gossip with heartbeat failure detection):
//! * **Rounds.** Every 100 ms (the first round at a random moment within the first 100 ms, drawn from the node's own
//!   seeded random generator) a node increments its own heartbeat counter, picks up to 2 distinct random peers and
//!   sends each of them its whole state. A node that receives such a push merges it and answers with its own whole
//!   state (the answer is not answered again). That is the only traffic: a node never broadcasts to everybody.
//! * **Data.** `put` writes a key locally with version = (highest version this node knows for the key) + 1, tagged
//!   with the writer's id. The winner for a key is the entry with the higher version; equal versions are decided by
//!   the larger writer id. Merging is therefore commutative, associative and idempotent, so every node ends with the
//!   same value once the gossip has spread.
//! * **Failure detection.** Heartbeat counters travel with the state and are merged by maximum. A node counts a peer
//!   as alive as long as the peer's counter has increased within the last 2000 ms of this node's own time; at the
//!   start every peer is given the benefit of the doubt (as if its counter had just increased). A peer that shows
//!   progress again becomes alive again.
//!
//! The payload format is yours.

pub trait GossipNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    /// Called once per millisecond with the current simulated time (milliseconds since the start).
    fn on_tick(&self, now_ms: u64);

    /// Writes locally; the value spreads by gossip.
    fn put(&self, key: &str, value: &str);

    /// The value this node currently believes wins for the key.
    fn get(&self, key: &str) -> Option<String>;

    /// This node itself plus every peer it currently believes to be alive, sorted.
    fn alive_members(&self) -> Vec<String>;
}
