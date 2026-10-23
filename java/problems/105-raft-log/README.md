# 105 · Raft Log Replication (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

We need a replicated log that keeps accepting commands as long as a majority of nodes is alive, and never loses or reorders a command once it is committed - the core of etcd, Consul or a replicated database. Implement one node of a Raft cluster: leader election and log replication from the Raft paper, without snapshots and without membership changes.

Clients send commands to whichever node they believe is the leader; a node that is not the leader refuses. The cluster runs in a simulated network that loses, duplicates, delays and reorders messages, crashes nodes (paused, memory kept) and splits into partitions. The tests check the Raft safety properties after EVERY simulated millisecond, so being right most of the time is not enough.

**In short:** one Raft node. It elects a leader, accepts commands on the leader with `propose`, replicates them to followers, and exposes the commands that are safely committed through `committed()`. The tests check after every simulated millisecond that no term has two leaders and that committed lists never change and never disagree between nodes.

## Requirements

1. Election: a follower that hears nothing for a randomly drawn timeout in [150, 300] ms (from the node's seeded generator, redrawn after every reset) becomes a candidate in a new term and asks for votes. One vote per term. A higher term seen in any message makes the node adopt it and become a FOLLOWER. A majority of votes makes the candidate LEADER.
2. Election restriction: a vote is granted only to a candidate whose log is at least as up to date as the voter's (higher last term wins, then the longer log).
3. A leader sends a heartbeat (AppendEntries) every 50 ms. Check-quorum: a leader that has had no AppendEntries replies from a majority within 300 ms steps down.
4. Log entries have a 1-based index and a term. A new leader appends one hidden no-op entry of its own term; no-ops take an index but never show up in `committed()`.
5. AppendEntries carries prevIndex/prevTerm, the entries from the follower's nextIndex on and the leader's commit index. A follower rejects it if it has no entry at prevIndex with prevTerm; otherwise it appends, deleting its own entries only where an entry CONFLICTS (same index, different term) - an old or duplicated AppendEntries must never shorten the log. The leader backs nextIndex off by one on rejection.
6. `propose(command)` on the leader appends the command and returns its index (the leader sends it out at once, at the latest with the next heartbeat); on a non-leader it returns empty. Accepted is not committed.
7. The leader commits index N when a majority (itself included) stores it AND the entry is from the leader's own term. A follower sets its commit index to min(leaderCommit, index of the last entry it verified). `committed()` lists the commands up to the commit index, oldest first, no-ops left out.
8. `role()`, `term()`, `leader()` (the leader known in the current term: itself if leader, empty if unknown) and `commitIndex()` (no-ops count in the index) expose the state to the tests.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. The Rust traits take `&self`.
- Time is simulated: one tick = 1 ms, `onTick(nowMs)` is called once per millisecond with the current time. Never read the wall clock, never sleep; derive every timeout from the `nowMs` you were handed.
- Network model: a message sent now is delivered after a latency of 1-5 ms by default (random per message, so messages can overtake each other). Tests can change the latency range, drop a fraction of the messages, duplicate a fraction, cut the network into partitions (messages across the cut are lost, including ones already in flight) and heal it.
- Crash model: a crashed node gets no ticks and no messages and its sends are ignored, but its memory is kept (it is paused, not wiped). After `recover` it simply continues with its first tick.
- Same seed, same run: any randomness of yours must come from the seed you are given, and iteration order of your collections must not influence behaviour (a replay is compared in the tests).
- The tests check, after every simulated millisecond, that there is at most one leader per term, that no node's committed list ever changes or shrinks, and that the committed lists of all nodes are prefixes of one another.
- A crashed node is only paused - it keeps its memory, so no disk is needed - and it sees the time jump on recovery (your timers must tolerate that and not fire a burst of elections).
- Faults injected: crashes and recoveries of leaders and followers, 25% loss, 20-50% duplication with latencies up to 60 ms (reordering), partitions of 2 vs 3 nodes, and random combinations over 100 seeds with 5 nodes.

## Examples

```
3 nodes, leader n1 (term 3).   n1.propose("a") -> returns index 2   (index 1 is n1's hidden no-op)
n1 -> n2,n3: AppendEntries(prev=(1,term3), entries=[a@2])   n2 acks   -> majority (n1+n2) -> commitIndex=2
next heartbeat carries leaderCommit=2 -> n2, n3 commit; committed() == ["a"] on all nodes
n1 partitioned alone: propose("doomed") accepted at index 3, never committed.
n2 elected term 4: appends its own no-op at index 3 (term 4), then commits "winner" at index 4 ->
on heal n1 steps down, its entry 3 ("doomed", term 3) is overwritten; committed() == ["a","winner"] everywhere.
```

Index versus command list (no-ops take an index but are hidden):

```
log (index:term:command)  1:t1:<no-op>  2:t1:"a"  3:t2:<no-op>  4:t2:"b"
commitIndex() == 4  ->  committed() == ["a", "b"]      commitIndex() == 2 -> committed() == ["a"]
```

Voting with logs (voter's log ends at index 5 in term 2):

```
candidate's last entry is (index 3, term 3)  -> granted: higher last term wins even though its log is shorter
candidate's last entry is (index 4, term 2)  -> refused: same last term but shorter
candidate's last entry is (index 5, term 2)  -> granted: same last term, same length ("at least as up to date")
(the term and one-vote-per-term rules of problem 99 apply on top of this)
```

Rejecting and backing off (leader log has entries 1..5; follower has 1..2 only; "t" stands for each entry's term):

```
leader first tries prev=(5,t) with no entries -> follower has no entry at index 5: rejects
leader lowers that follower's nextIndex by one and retries: prev=(4,t) with entries [5]     -> rejected
                                                           prev=(3,t) with entries [4,5]   -> rejected
                                                           prev=(2,t) with entries [3,4,5] -> prev matches: follower appends 3,4,5 and acknowledges
```

Conflicts, and stale AppendEntries that must not shorten the log:

```
follower log: 1:t1  2:t2 (written by an old leader)  3:t2
new leader (term 3) sends prev=(1,t1), entries [2:t3]
  -> index 2 exists with a different term: CONFLICT, so entries 2 and 3 are deleted and 2:t3 is appended
a delayed duplicate AppendEntries carrying only prev=(1,t1), entries [2:t3] arrives when the follower already has 1:t1 2:t3 3:t3
  -> every carried entry already matches: nothing is deleted, entry 3 stays
```

Commit only counts own-term entries:

```
leader n1 in term 4 holds entry 2 (term 3) on a majority, and its new no-op 3 (term 4) only on itself
  -> commitIndex stays below 2 (an older-term entry is not committed by counting replicas)
no-op 3 reaches a majority -> commitIndex becomes 3; entries 2 and 3 commit together
```

## Edge cases

- `propose` returns empty on a FOLLOWER or CANDIDATE; on the leader it returns the index of the new entry. Consecutive proposals on the same leader (same term) return consecutive indexes. Accepted is not committed.
- A node that lost leadership (step-down or higher term) keeps its uncommitted entries only until a new leader's log overwrites them; they never show up in `committed()` if they were overwritten.
- A follower's commit index is min(leaderCommit, index of the last entry it verified in this AppendEntries); a heartbeat claiming leaderCommit = 10 to a follower that only verified up to index 6 moves it to 6. A commit index never decreases.
- Check-quorum: replies from a majority (counting the leader itself) must have arrived within the last 300 ms, otherwise the leader steps down. A leader isolated from its followers cannot commit and stops being leader within about 700 ms of the isolation.
- A restarted/recovered node sees time jump forward; it must not start a burst of elections because of that.

## Glossary

- **Term**: election round counter; higher terms override lower ones.
- **Log entry / index**: a command tagged with the term it was created in and a 1-based position in the log.
- **No-op**: a placeholder entry a new leader appends at the start of its term; it counts in indexes, not in `committed()`.
- **AppendEntries**: the leader's message that both replicates entries and acts as heartbeat; it names the entry just before the new ones (prevIndex, prevTerm).
- **nextIndex**: the leader's guess of the next index to send to a given follower; lowered after a rejection.
- **Commit index**: the highest index known to be safely stored on a majority (of a current-term entry); everything up to it is final.
- **Up to date**: log comparison for votes: the log whose last entry has the higher term is newer; with equal last terms the longer log is newer.
- **Deposed leader**: a leader that has learned of a higher term or lost its majority.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.create(id, peers, send, seed)` with `peers` the OTHER nodes; `Role.{FOLLOWER, CANDIDATE, LEADER}`; `leader()` returns `Optional<String>`; `propose` returns `Optional<Long>`. Rust: `create(id, peers, send, seed)`; `Role::{Follower, Candidate, Leader}`, `term() -> u64`, `leader() -> Option<String>`, `propose -> Option<u64>`, `commit_index() -> u64`.
- Timers: election timeout drawn uniformly from [150, 300] ms from the node's seeded generator and redrawn after every reset (hearing from a valid leader or granting a vote); heartbeat every 50 ms; check-quorum window 300 ms.
- Election rules as in problem 99 (one vote per term, higher term => adopt and become FOLLOWER) plus the log-up-to-date check on RequestVote.
- The leader appends exactly one no-op (term = own term) on election; it occupies an index. The first client entry therefore has index >= 2 on a fresh cluster and consecutive proposals return consecutive indexes.
- A leader never commits an entry from an earlier term by counting replicas; it commits it indirectly by committing an entry of its own term after it.
- A follower deletes log entries only on a real conflict; a stale or duplicated AppendEntries (shorter than the current log) leaves a longer matching log intact. Its commit index = min(leaderCommit, index of the last new entry checked), and it never decreases.
- `propose` is refused (empty) on a follower or candidate. A deposed leader's uncommitted entries may be overwritten and must then not show up in `committed()`.
- The payload format is yours; the run is reproducible from the seed.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. One instance is ONE node of a Raft cluster; the tests run the nodes inside a deterministic
 * simulation (see {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with
 * {@link #onTick} (once per simulated millisecond) and read the state through the getters. Nodes may only communicate
 * through the {@code send} function they were created with. Single-threaded: no locking needed. A crashed node is
 * only paused (it keeps its memory, so you need no disk), messages to it are lost, and it sees time jump on recovery.
 *
 * <p>This is the Raft paper (leader election + log replication, no snapshots, no membership changes):
 * <ul>
 *   <li><b>Election</b> exactly as in the leader-election problem: election timeout drawn from [150, 300] ms from the
 *       node's seeded generator and redrawn after every reset, one vote per term, higher term => adopt it and become
 *       FOLLOWER, heartbeat every 50 ms, check-quorum (a leader without AppendEntries replies from a majority within
 *       300 ms steps down). In addition the <b>election restriction</b>: a vote is only granted to a candidate whose
 *       log is at least as up to date as the voter's (higher last term wins, then the longer log).</li>
 *   <li><b>Log.</b> Entries have a 1-based index and a term. A new leader appends one hidden no-op entry (term = its
 *       term) so that earlier entries can commit; no-ops take an index but never show up in {@link #committed()}.</li>
 *   <li><b>Replication.</b> The heartbeat is an AppendEntries with prevIndex/prevTerm, the entries from the follower's
 *       nextIndex on and the leader's commit index. A follower rejects it if its log has no entry at prevIndex with
 *       prevTerm; otherwise it appends the entries, deleting its own entries only where an entry CONFLICTS (same index,
 *       different term) - an old or duplicated AppendEntries must never shorten the log. The leader backs nextIndex off
 *       by one on a rejection. A leader that accepted {@link #propose} sends the new entry out at once (at the latest
 *       with the next heartbeat).</li>
 *   <li><b>Commit.</b> The leader commits index N when a majority (itself included) stores it AND the entry is from the
 *       leader's own term (earlier entries commit indirectly). A follower sets its commit index to
 *       min(leaderCommit, index of the last entry it verified).</li>
 * </ul>
 * Messages may be lost, duplicated, delayed and reordered. The payload format is yours.
 */
public interface RaftNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    Role role();

    long term();

    /** The leader this node knows in its current term: itself if it is the leader, empty if unknown. */
    Optional<String> leader();

    /**
     * Appends a client command if this node is the leader and returns the index of the new entry; empty if it is not
     * the leader. Accepted is not committed: a deposed leader's entries may be overwritten.
     */
    Optional<Long> propose(String command);

    /** Highest index known to be committed (no-ops count in the index). */
    long commitIndex();

    /** The commands of all committed entries, oldest first, no-ops left out. */
    List<String> committed();
}

package contract;

public enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
    public static RaftNode create(String id, List<String> peers, BiConsumer<String, String> send, long seed)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- A single leader is elected and every node knows its id; a follower refuses `propose`.
- Commands replicate and commit in order on all nodes; indexes are increasing and the no-op is hidden (first index >= 2, consecutive afterwards).
- A leader isolated from its followers cannot commit and steps down by check-quorum within 700 ms.
- Committed entries survive a leader crash; the new leader continues the log, the old one rejoins as FOLLOWER and catches up.
- A deposed leader's uncommitted entry is discarded after the partition heals; all nodes end with the same committed list.
- A follower that was down for 30 entries is brought up to date by log backtracking.
- A node with an outdated log (it missed two committed entries) never becomes leader even after the other nodes' leader dies (election restriction).
- Duplication (30%) with latencies up to 40 ms on 5 nodes, and 25% loss on 3 nodes keep safety and eventually commit everything.
- 100 random schedules of crashes, recoveries, partitions, loss and duplication: no violation of the per-millisecond safety checks; after healing the cluster commits a new command and all logs are identical.
- More edge cases (100 commands in a 5-node cluster, rolling restarts, heavy faults) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Log indexes are 1-based. Election timeouts come from the seed given to the constructor. The tests call getters from an observer after every millisecond.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/105-raft-log test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
