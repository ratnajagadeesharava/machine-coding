package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.RaftNode;
import contract.Role;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class BehaviorTest {

    /** A cluster in the simulation with the Raft safety invariants checked after every simulated millisecond. */
    private static final class Cluster {
        final SimNet net;
        final List<String> ids = new ArrayList<>();
        final Map<String, RaftNode> nodes = new HashMap<>();
        final Map<Long, String> leaderOfTerm = new HashMap<>();
        final Map<String, List<String>> lastCommitted = new HashMap<>();
        final List<String> violations = new ArrayList<>();
        List<String> longest = new ArrayList<>();

        Cluster(int size, long seed) {
            net = new SimNet(seed);
            for (int i = 0; i < size; i++) ids.add("n" + i);
            for (String id : ids) {
                List<String> peers = new ArrayList<>(ids);
                peers.remove(id);
                RaftNode node = Candidate.of(() -> Solution.create(id, peers, net.sender(id), seed * 31 + id.hashCode()));
                nodes.put(id, node);
                lastCommitted.put(id, List.of());
                net.register(id, node::onMessage, node::onTick);
            }
            net.observe(this::checkSafety);
        }

        void checkSafety() {
            for (String id : ids) {
                RaftNode n = nodes.get(id);
                if (n.role() == Role.LEADER) {
                    String previous = leaderOfTerm.putIfAbsent(n.term(), id);
                    if (previous != null && !previous.equals(id)) violations.add("two leaders in term " + n.term() + ": " + previous + ", " + id);
                }
                List<String> now = n.committed();
                List<String> before = lastCommitted.get(id);
                if (now.size() < before.size() || !now.subList(0, before.size()).equals(before)) {
                    violations.add(id + " changed its committed entries: " + before + " -> " + now + " at t=" + net.now());
                }
                lastCommitted.put(id, now);
                int common = Math.min(now.size(), longest.size());
                if (!now.subList(0, common).equals(longest.subList(0, common))) {
                    violations.add(id + " committed " + now + " but another node committed " + longest);
                }
                if (now.size() > longest.size()) longest = now;
            }
        }

        Optional<String> leader() {
            List<String> l = new ArrayList<>();
            for (String id : ids) if (!net.isCrashed(id) && nodes.get(id).role() == Role.LEADER) l.add(id);
            if (l.size() != 1) return Optional.empty();
            return Optional.of(l.get(0));
        }

        String awaitLeader(long maxMs) {
            net.runUntil(() -> leader().isPresent(), maxMs);
            return leader().orElseThrow(() -> new AssertionError("no single leader after " + maxMs + " ms"));
        }

        /** Proposes on the current leader; returns the index or throws. */
        long propose(String command) {
            String l = awaitLeader(2_000);
            return nodes.get(l).propose(command).orElseThrow();
        }

        boolean everyoneCommitted(List<String> expected) {
            return ids.stream().filter(i -> !net.isCrashed(i)).allMatch(i -> nodes.get(i).committed().equals(expected));
        }

        List<String> committedOn(String id) {
            return nodes.get(id).committed();
        }
    }

    @Test
    void shouldElectOneLeaderAndEveryoneKnowsIt() {
        Cluster c = new Cluster(3, 1);

        String leader = c.awaitLeader(1_000);
        c.net.run(100);

        for (String id : c.ids) assertThat(c.nodes.get(id).leader()).as(id).hasValue(leader);
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldRefuseToProposeOnAFollower() {
        Cluster c = new Cluster(3, 2);
        String leader = c.awaitLeader(1_000);
        String follower = c.ids.stream().filter(i -> !i.equals(leader)).findFirst().orElseThrow();

        assertThat(c.nodes.get(follower).propose("x")).isEmpty();
        assertThat(c.nodes.get(leader).propose("x")).isPresent();
    }

    @Test
    void shouldReplicateAndCommitCommandsInOrderOnEveryNode() {
        Cluster c = new Cluster(3, 3);
        c.awaitLeader(1_000);

        c.propose("a");
        c.propose("b");
        c.propose("c");

        assertThat(c.net.runUntil(() -> c.everyoneCommitted(List.of("a", "b", "c")), 1_000)).isTrue();
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldReturnIncreasingIndexesAndHideTheNoOp() {
        Cluster c = new Cluster(3, 4);
        long first = c.propose("a");
        long second = c.propose("b");

        assertThat(first).isGreaterThanOrEqualTo(2); // the leader's no-op took index 1
        assertThat(second).isEqualTo(first + 1);
        c.net.run(300);
        assertThat(c.committedOn(c.awaitLeader(500))).containsExactly("a", "b");
    }

    @Test
    void shouldNotCommitWithoutAMajority() {
        Cluster c = new Cluster(3, 5);
        String leader = c.awaitLeader(1_000);
        c.net.run(100);
        for (String id : c.ids) if (!id.equals(leader)) c.net.crash(id);

        assertThat(c.nodes.get(leader).propose("lonely")).isPresent();
        c.net.run(700);

        assertThat(c.committedOn(leader)).isEmpty();
        assertThat(c.nodes.get(leader).role()).isNotEqualTo(Role.LEADER); // check-quorum
    }

    @Test
    void shouldKeepCommittedEntriesAcrossALeaderFailure() {
        Cluster c = new Cluster(3, 6);
        String old = c.awaitLeader(1_000);
        c.propose("a");
        c.propose("b");
        c.net.runUntil(() -> c.everyoneCommitted(List.of("a", "b")), 1_000);

        c.net.crash(old);
        String next = c.awaitLeader(2_000);
        c.propose("c");

        assertThat(next).isNotEqualTo(old);
        assertThat(c.net.runUntil(() -> c.everyoneCommitted(List.of("a", "b", "c")), 1_000)).isTrue();

        c.net.recover(old);
        assertThat(c.net.runUntil(() -> c.everyoneCommitted(List.of("a", "b", "c")), 2_000)).isTrue();
        assertThat(c.nodes.get(old).role()).isEqualTo(Role.FOLLOWER);
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldDiscardTheUncommittedEntriesOfADeposedLeader() {
        Cluster c = new Cluster(3, 7);
        String old = c.awaitLeader(1_000);
        c.propose("kept");
        c.net.runUntil(() -> c.everyoneCommitted(List.of("kept")), 1_000);
        List<String> others = new ArrayList<>(c.ids);
        others.remove(old);

        c.net.partition(List.of(old), others);
        assertThat(c.nodes.get(old).propose("doomed")).isPresent();
        c.net.runUntil(() -> others.stream().anyMatch(i -> c.nodes.get(i).role() == Role.LEADER), 2_000);
        String fresh = others.stream().filter(i -> c.nodes.get(i).role() == Role.LEADER).findFirst().orElseThrow();
        c.nodes.get(fresh).propose("winner");
        c.net.run(300);
        c.net.heal();

        assertThat(c.net.runUntil(() -> c.everyoneCommitted(List.of("kept", "winner")), 3_000)).isTrue();
        assertThat(c.committedOn(old)).doesNotContain("doomed");
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldCatchUpAFollowerThatMissedManyEntries() {
        Cluster c = new Cluster(3, 8);
        String leader = c.awaitLeader(1_000);
        String slow = c.ids.stream().filter(i -> !i.equals(leader)).findFirst().orElseThrow();
        c.net.crash(slow);
        List<String> expected = new ArrayList<>();
        for (int i = 0; i < 30; i++) {
            c.propose("c" + i);
            expected.add("c" + i);
            c.net.run(20);
        }
        c.net.recover(slow);

        assertThat(c.net.runUntil(() -> c.everyoneCommitted(expected), 3_000)).isTrue();
    }

    @Test
    void shouldNeverElectANodeWithAnOutdatedLog() {
        Cluster c = new Cluster(3, 9);
        String leader = c.awaitLeader(1_000);
        List<String> others = new ArrayList<>(c.ids);
        others.remove(leader);
        String stale = others.get(0);
        String fresh = others.get(1);
        c.net.crash(stale);
        c.propose("a");
        c.propose("b");
        c.net.runUntil(() -> c.nodes.get(fresh).committed().equals(List.of("a", "b")), 1_000);
        c.net.crash(leader);
        c.net.run(20); // let the heartbeats that were already on the wire arrive (or be lost) while the stale node is still down
        c.net.recover(stale);

        c.net.run(3_000);

        assertThat(c.nodes.get(stale).role()).isNotEqualTo(Role.LEADER);
        assertThat(c.committedOn(fresh)).containsExactly("a", "b");
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldStayConsistentWithDuplicatedAndReorderedMessages() {
        Cluster c = new Cluster(5, 10);
        c.net.setDuplicateRate(0.3);
        c.net.setLatency(1, 40);
        List<String> expected = new ArrayList<>();
        for (int i = 0; i < 20; i++) {
            String l = c.awaitLeader(3_000);
            if (c.nodes.get(l).propose("c" + i).isPresent()) expected.add("c" + i);
            c.net.run(30);
        }

        c.net.run(2_000);

        assertThat(c.violations).isEmpty();
        assertThat(c.committedOn(c.awaitLeader(1_000))).containsSubsequence(expected.subList(0, 1));
    }

    @Test
    void shouldCommitEverythingWhenTheNetworkLosesMessages() {
        Cluster c = new Cluster(3, 11);
        c.net.setDropRate(0.25);
        c.awaitLeader(3_000);
        List<String> expected = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            c.propose("c" + i);
            expected.add("c" + i);
            c.net.run(200);
        }

        assertThat(c.net.runUntil(() -> c.everyoneCommitted(expected), 5_000)).isTrue();
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldNeverBreakSafetyUnderRandomCrashesAndPartitions() {
        for (long seed = 0; seed < 100; seed++) {
            Cluster c = new Cluster(5, seed);
            Random rnd = new Random(seed * 17 + 3);
            c.net.setDropRate(0.05);
            c.net.setDuplicateRate(0.2);
            c.net.setLatency(1, 30);
            int proposed = 0;
            for (int round = 0; round < 25; round++) {
                switch (rnd.nextInt(5)) {
                    case 0 -> c.net.crash(c.ids.get(rnd.nextInt(5)));
                    case 1 -> c.ids.forEach(c.net::recover);
                    case 2 -> c.net.partition(List.of("n0", "n1"), List.of("n2", "n3", "n4"));
                    case 3 -> c.net.heal();
                    default -> { }
                }
                for (String id : c.ids) {
                    if (!c.net.isCrashed(id) && c.nodes.get(id).propose("s" + seed + "-" + proposed).isPresent()) proposed++;
                }
                c.net.run(50 + rnd.nextInt(250));
            }
            assertThat(c.violations).as("seed %d", seed).isEmpty();

            c.net.heal();
            c.ids.forEach(c.net::recover);
            boolean committed = false;
            for (int attempt = 0; attempt < 5 && !committed; attempt++) {
                // a proposal can be lost if the leader is deposed right away: that is legal Raft, so ask again
                String cmd = "final" + attempt;
                c.nodes.get(c.awaitLeader(3_000)).propose(cmd);
                committed = c.net.runUntil(() -> c.ids.stream().allMatch(i -> c.nodes.get(i).committed().contains(cmd)), 1_500);
            }
            assertThat(committed).as("seed %d: the healed cluster must commit", seed).isTrue();
            assertThat(c.ids.stream().map(c::committedOn).distinct().count()).as("seed %d: all logs equal", seed).isEqualTo(1);
            assertThat(c.violations).as("seed %d", seed).isEmpty();
        }
    }
}
