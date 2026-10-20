package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ElectionNode;
import contract.Role;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class BehaviorTest {

    /** A cluster inside the simulation, with the election-safety invariant checked after every simulated millisecond. */
    private static final class Cluster {
        final SimNet net;
        final List<String> ids = new ArrayList<>();
        final Map<String, ElectionNode> nodes = new HashMap<>();
        final Map<Long, String> leaderOfTerm = new HashMap<>();
        final List<String> violations = new ArrayList<>();

        Cluster(int size, long seed) {
            net = new SimNet(seed);
            for (int i = 0; i < size; i++) {
                ids.add("n" + i);
            }
            for (String id : ids) {
                List<String> peers = new ArrayList<>(ids);
                peers.remove(id);
                ElectionNode node = Candidate.of(() -> Solution.create(id, peers, net.sender(id), seed * 31 + id.hashCode()));
                nodes.put(id, node);
                net.register(id, node::onMessage, node::onTick);
            }
            net.observe(this::checkSafety);
        }

        void checkSafety() {
            for (String id : ids) {
                ElectionNode n = nodes.get(id);
                if (n.role() == Role.LEADER) {
                    String previous = leaderOfTerm.putIfAbsent(n.term(), id);
                    if (previous != null && !previous.equals(id)) {
                        violations.add("two leaders in term " + n.term() + ": " + previous + " and " + id + " at t=" + net.now());
                    }
                }
            }
        }

        List<String> leaders() {
            List<String> out = new ArrayList<>();
            for (String id : ids) {
                if (!net.isCrashed(id) && nodes.get(id).role() == Role.LEADER) {
                    out.add(id);
                }
            }
            return out;
        }

        boolean hasSingleLeaderAmong(Set<String> members) {
            List<String> l = new ArrayList<>();
            for (String id : members) {
                if (nodes.get(id).role() == Role.LEADER) {
                    l.add(id);
                }
            }
            return l.size() == 1;
        }

        boolean allAgree(Set<String> members) {
            String leader = null;
            long term = -1;
            for (String id : members) {
                ElectionNode n = nodes.get(id);
                if (n.leader().isEmpty()) {
                    return false;
                }
                if (leader == null) {
                    leader = n.leader().get();
                    term = n.term();
                } else if (!leader.equals(n.leader().get()) || term != n.term()) {
                    return false;
                }
            }
            return true;
        }

        Set<String> all() {
            return new java.util.LinkedHashSet<>(ids);
        }
    }

    @Test
    void shouldElectExactlyOneLeaderInClustersOfThreeFiveAndSeven() {
        for (int size : new int[] {3, 5, 7}) {
            Cluster c = new Cluster(size, 100 + size);

            boolean elected = c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
            c.net.run(500);

            assertThat(elected).as("%d nodes elect a leader", size).isTrue();
            assertThat(c.leaders()).hasSize(1);
            assertThat(c.allAgree(c.all())).as("%d nodes agree on leader and term", size).isTrue();
            assertThat(c.violations).isEmpty();
        }
    }

    @Test
    void shouldElectItselfWhenAloneInTheCluster() {
        Cluster c = new Cluster(1, 1);

        boolean elected = c.net.runUntil(() -> c.leaders().size() == 1, 600);

        assertThat(elected).isTrue();
        assertThat(c.nodes.get("n0").leader()).hasValue("n0");
    }

    @Test
    void shouldKeepTheSameLeaderAndTermWhileNothingFails() {
        Cluster c = new Cluster(5, 7);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        c.net.run(500);
        String leader = c.leaders().get(0);
        long term = c.nodes.get(leader).term();

        c.net.run(10_000);

        assertThat(c.leaders()).containsExactly(leader);
        assertThat(c.nodes.get(leader).term()).as("no unnecessary elections").isEqualTo(term);
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldElectANewLeaderWithAHigherTermWhenTheLeaderCrashes() {
        Cluster c = new Cluster(5, 11);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        String oldLeader = c.leaders().get(0);
        long oldTerm = c.nodes.get(oldLeader).term();

        c.net.crash(oldLeader);
        boolean replaced = c.net.runUntil(() -> c.leaders().size() == 1 && !c.leaders().get(0).equals(oldLeader), 1_500);

        assertThat(replaced).isTrue();
        assertThat(c.nodes.get(c.leaders().get(0)).term()).isGreaterThan(oldTerm);
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldMakeARecoveredOldLeaderStepDown() {
        Cluster c = new Cluster(5, 13);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        String oldLeader = c.leaders().get(0);
        c.net.crash(oldLeader);
        c.net.runUntil(() -> c.leaders().size() == 1 && !c.leaders().get(0).equals(oldLeader), 1_500);
        String newLeader = c.leaders().get(0);

        c.net.recover(oldLeader);
        c.net.run(600);

        assertThat(c.nodes.get(oldLeader).role()).isEqualTo(Role.FOLLOWER);
        assertThat(c.nodes.get(oldLeader).leader()).hasValue(newLeader);
        assertThat(c.leaders()).containsExactly(newLeader);
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldNotElectAnythingWithoutAMajority() {
        Cluster c = new Cluster(5, 17);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        List<String> followers = new ArrayList<>(c.ids);
        followers.remove(c.leaders().get(0));
        c.net.crash(c.leaders().get(0));
        c.net.crash(followers.get(0));
        c.net.crash(followers.get(1)); // 2 of 5 left up... then 3 left: crash one more
        c.net.crash(followers.get(2));

        c.net.run(3_000);

        assertThat(c.leaders()).as("a single survivor of five can never win").isEmpty();
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldLetTheMajoritySideElectAndStripTheMinorityLeaderOfItsLeadership() {
        Cluster c = new Cluster(5, 19);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        String oldLeader = c.leaders().get(0);
        List<String> minority = new ArrayList<>(List.of(oldLeader));
        List<String> majority = new ArrayList<>();
        for (String id : c.ids) {
            if (!id.equals(oldLeader)) {
                (minority.size() < 2 ? minority : majority).add(id);
            }
        }

        c.net.partition(minority, majority);
        c.net.run(1_500);

        assertThat(c.hasSingleLeaderAmong(new java.util.HashSet<>(majority))).as("the majority elects a leader").isTrue();
        for (String id : minority) {
            assertThat(c.nodes.get(id).role()).as("%s in the minority", id).isNotEqualTo(Role.LEADER);
        }
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldConvergeToOneLeaderAndOneTermAfterAPartitionHeals() {
        Cluster c = new Cluster(5, 23);
        c.net.runUntil(() -> c.leaders().size() == 1, 1_500);
        c.net.partition(List.of("n0", "n1"), List.of("n2", "n3", "n4"));
        c.net.run(2_000);

        c.net.heal();
        boolean converged = c.net.runUntil(() -> c.leaders().size() == 1 && c.allAgree(c.all()), 2_000);

        assertThat(converged).isTrue();
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldStillElectWithTwentyPercentMessageLoss() {
        Cluster c = new Cluster(5, 29);
        c.net.setDropRate(0.2);

        boolean elected = c.net.runUntil(() -> c.leaders().size() == 1, 4_000);
        c.net.run(3_000);

        assertThat(elected).isTrue();
        assertThat(c.violations).isEmpty();
    }

    @Test
    void shouldReplayTheSameRunFromTheSameSeed() {
        List<String> runs = new ArrayList<>();
        for (int i = 0; i < 2; i++) {
            Cluster c = new Cluster(5, 31);
            c.net.run(3_000);
            runs.add(c.leaders() + "@" + c.nodes.get(c.ids.get(0)).term());
        }

        assertThat(runs.get(1)).isEqualTo(runs.get(0));
    }

    @Test
    void shouldKeepElectionSafetyOver100RandomFaultSchedules() {
        for (int seed = 0; seed < 100; seed++) {
            Cluster c = new Cluster(5, 1_000 + seed);
            java.util.Random faults = new java.util.Random(seed);
            for (int round = 0; round < 12; round++) {
                c.net.run(150 + faults.nextInt(400));
                String victim = c.ids.get(faults.nextInt(5));
                switch (faults.nextInt(4)) {
                    case 0 -> c.net.crash(victim);
                    case 1 -> c.net.recover(victim);
                    case 2 -> c.net.partition(List.of(victim), c.ids.stream().filter(x -> !x.equals(victim)).toList());
                    default -> c.net.heal();
                }
            }
            c.net.heal();
            for (String id : c.ids) {
                c.net.recover(id);
            }
            boolean settled = c.net.runUntil(() -> c.leaders().size() == 1 && c.allAgree(c.all()), 3_000);

            assertThat(c.violations).as("seed %d", seed).isEmpty();
            assertThat(settled).as("seed %d settles after faults end", seed).isTrue();
        }
    }
}
