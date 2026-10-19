//! Thin test entry point. One instance is ONE node of a Raft cluster; the tests run the nodes inside a deterministic
//! simulation (`playground_support::SimNet`): they deliver messages with `on_message`, advance time with `on_tick`
//! (once per simulated millisecond) and read the state through the getters. Nodes may only communicate through the
//! `send` function they were created with. Use interior mutability (`Mutex`) for the state: the trait takes `&self`.
//! A crashed node is only paused (it keeps its memory, so you need no disk), messages to it are lost, and it sees time
//! jump on recovery.
//!
//! This is the Raft paper (leader election + log replication, no snapshots, no membership changes):
//! * **Election** exactly as in the leader-election problem: election timeout drawn from [150, 300] ms from the
//!   node's seeded generator and redrawn after every reset, one vote per term, higher term => adopt it and become
//!   follower, heartbeat every 50 ms, check-quorum (a leader without AppendEntries replies from a majority within
//!   300 ms steps down). In addition the **election restriction**: a vote is only granted to a candidate whose log is
//!   at least as up to date as the voter's (higher last term wins, then the longer log).
//! * **Log.** Entries have a 1-based index and a term. A new leader appends one hidden no-op entry (term = its term)
//!   so that earlier entries can commit; no-ops take an index but never show up in `committed()`.
//! * **Replication.** The heartbeat is an AppendEntries with prevIndex/prevTerm, the entries from the follower's
//!   nextIndex on and the leader's commit index. A follower rejects it if its log has no entry at prevIndex with
//!   prevTerm; otherwise it appends the entries, deleting its own entries only where an entry CONFLICTS (same index,
//!   different term) - an old or duplicated AppendEntries must never shorten the log. The leader backs nextIndex off
//!   by one on a rejection. A leader that accepted `propose` sends the new entry out at once (at the latest with the
//!   next heartbeat).
//! * **Commit.** The leader commits index N when a majority (itself included) stores it AND the entry is from the
//!   leader's own term (earlier entries commit indirectly). A follower sets its commit index to
//!   min(leaderCommit, index of the last entry it verified).
//!
//! Messages may be lost, duplicated, delayed and reordered. The payload format is yours.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub trait RaftNode: Send + Sync {
    fn on_message(&self, from: &str, payload: &str);

    /// Called once per millisecond with the current simulated time (milliseconds since the start).
    fn on_tick(&self, now_ms: u64);

    fn role(&self) -> Role;

    fn term(&self) -> u64;

    /// The leader this node knows in its current term: itself if it is the leader, `None` if unknown.
    fn leader(&self) -> Option<String>;

    /// Appends a client command if this node is the leader and returns the index of the new entry; `None` if it is
    /// not the leader. Accepted is not committed: a deposed leader's entries may be overwritten.
    fn propose(&self, command: &str) -> Option<u64>;

    /// Highest index known to be committed (no-ops count in the index).
    fn commit_index(&self) -> u64;

    /// The commands of all committed entries, oldest first, no-ops left out.
    fn committed(&self) -> Vec<String>;
}
