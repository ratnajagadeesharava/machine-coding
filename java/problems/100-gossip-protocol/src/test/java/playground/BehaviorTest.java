package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.GossipNode;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class BehaviorTest {

    private static final class Cluster {
        final SimNet net;
        final List<String> ids = new ArrayList<>();
        final Map<String, GossipNode> nodes = new HashMap<>();

        Cluster(int size, long seed) {
            net = new SimNet(seed);
            for (int i = 0; i < size; i++) {
                ids.add("n" + i);
            }
            for (String id : ids) {
                List<String> peers = new ArrayList<>(ids);
                peers.remove(id);
                GossipNode node = Candidate.of(() -> Solution.create(id, peers, net.sender(id), seed * 131 + id.hashCode()));
                nodes.put(id, node);
                net.register(id, node::onMessage, node::onTick);
            }
        }

        boolean everyoneHas(String key, String value) {
            for (String id : ids) {
                if (!net.isCrashed(id) && !nodes.get(id).get(key).equals(Optional.of(value))) {
                    return false;
                }
            }
            return true;
        }

        boolean agreeOn(String key) {
            Optional<String> first = null;
            for (String id : ids) {
                if (net.isCrashed(id)) {
                    continue;
                }
                Optional<String> v = nodes.get(id).get(key);
                if (v.isEmpty() || (first != null && !first.equals(v))) {
                    return false;
                }
                first = v;
            }
            return true;
        }

        Set<String> aliveSeenBy(String id) {
            return nodes.get(id).aliveMembers();
        }
    }

    @Test
    void shouldSpreadAWriteToEveryNodeInLogarithmicallyManyRounds() {
        Cluster c = new Cluster(16, 1);
        c.nodes.get("n0").put("color", "blue");

        boolean spread = c.net.runUntil(() -> c.everyoneHas("color", "blue"), 1_500);

        assertThat(spread).as("16 nodes, fanout 2, 100 ms rounds: well within 15 rounds").isTrue();
    }

    @Test
    void shouldSpreadManyKeysFromManyWriters() {
        Cluster c = new Cluster(12, 2);
        for (int i = 0; i < 12; i++) {
            c.nodes.get("n" + i).put("key-" + i, "value-" + i);
        }

        c.net.run(2_500);

        for (int i = 0; i < 12; i++) {
            assertThat(c.everyoneHas("key-" + i, "value-" + i)).as("key-%d", i).isTrue();
        }
    }

    @Test
    void shouldLetALaterWriteOverwriteAnEarlierOneEverywhere() {
        Cluster c = new Cluster(10, 3);
        c.nodes.get("n0").put("k", "first");
        c.net.runUntil(() -> c.everyoneHas("k", "first"), 2_000);

        c.nodes.get("n7").put("k", "second");
        boolean spread = c.net.runUntil(() -> c.everyoneHas("k", "second"), 2_000);

        assertThat(spread).isTrue();
        c.net.run(1_000);
        assertThat(c.everyoneHas("k", "second")).as("the old value must never come back").isTrue();
    }

    @Test
    void shouldResolveConcurrentWritesToTheSameValueEverywhere() {
        Cluster c = new Cluster(8, 4);
        c.nodes.get("n2").put("k", "from-n2");
        c.nodes.get("n5").put("k", "from-n5"); // same version: the larger writer id wins

        c.net.run(2_000);

        assertThat(c.agreeOn("k")).isTrue();
        assertThat(c.nodes.get("n0").get("k")).hasValue("from-n5");
    }

    @Test
    void shouldStillConvergeWithThirtyPercentMessageLoss() {
        Cluster c = new Cluster(16, 5);
        c.net.setDropRate(0.3);
        c.nodes.get("n3").put("k", "v");

        boolean spread = c.net.runUntil(() -> c.everyoneHas("k", "v"), 5_000);

        assertThat(spread).isTrue();
    }

    @Test
    void shouldConvergeWithDuplicatedAndReorderedMessages() {
        Cluster c = new Cluster(10, 6);
        c.net.setDuplicateRate(0.5);
        c.net.setLatency(1, 120);
        c.nodes.get("n1").put("k", "a");
        c.nodes.get("n8").put("k2", "b");

        c.net.run(4_000);

        assertThat(c.everyoneHas("k", "a")).isTrue();
        assertThat(c.everyoneHas("k2", "b")).isTrue();
    }

    @Test
    void shouldNotSuspectAnyoneInASteadyHealthyCluster() {
        Cluster c = new Cluster(16, 7);

        c.net.run(10_000);

        for (String id : c.ids) {
            assertThat(c.aliveSeenBy(id)).as("alive set of %s", id).hasSize(16);
        }
    }

    @Test
    void shouldNotSuspectAnyoneWhenAFifthOfTheMessagesAreLost() {
        Cluster c = new Cluster(16, 8);
        c.net.setDropRate(0.2);

        c.net.run(10_000);

        for (String id : c.ids) {
            assertThat(c.aliveSeenBy(id)).as("alive set of %s", id).hasSize(16);
        }
    }

    @Test
    void shouldDetectACrashedNodeEverywhereAfterTheTimeout() {
        Cluster c = new Cluster(16, 9);
        c.net.run(1_000);

        c.net.crash("n3");
        c.net.run(1_500);
        for (String id : c.ids) {
            if (!id.equals("n3")) {
                assertThat(c.aliveSeenBy(id)).as("%s should not suspect n3 yet (under 2 s)", id).contains("n3");
            }
        }
        c.net.run(2_000);

        for (String id : c.ids) {
            if (!id.equals("n3")) {
                assertThat(c.aliveSeenBy(id)).as("%s must have noticed", id).doesNotContain("n3").hasSize(15);
            }
        }
    }

    @Test
    void shouldReadmitARecoveredNodeAndCatchItUpOnMissedWrites() {
        Cluster c = new Cluster(12, 10);
        c.net.run(500);
        c.net.crash("n4");
        c.nodes.get("n0").put("missed", "while-away");
        c.net.run(4_000);

        c.net.recover("n4");
        boolean caughtUp = c.net.runUntil(() -> c.nodes.get("n4").get("missed").equals(Optional.of("while-away")), 2_000);
        c.net.run(1_500);

        assertThat(caughtUp).isTrue();
        for (String id : c.ids) {
            assertThat(c.aliveSeenBy(id)).as("alive set of %s", id).hasSize(12);
        }
    }

    @Test
    void shouldSplitIntoTwoViewsDuringAPartitionAndMergeAfterItHeals() {
        Cluster c = new Cluster(8, 11);
        c.net.run(500);
        List<String> left = List.of("n0", "n1", "n2", "n3");
        List<String> right = List.of("n4", "n5", "n6", "n7");

        c.net.partition(left, right);
        c.nodes.get("n0").put("left-key", "L");
        c.nodes.get("n7").put("right-key", "R");
        c.net.run(4_000);

        assertThat(c.nodes.get("n1").get("right-key")).isEmpty();
        assertThat(c.aliveSeenBy("n1")).containsExactlyInAnyOrderElementsOf(left);
        assertThat(c.aliveSeenBy("n6")).containsExactlyInAnyOrderElementsOf(right);

        c.net.heal();
        c.net.run(3_000);

        assertThat(c.everyoneHas("left-key", "L")).isTrue();
        assertThat(c.everyoneHas("right-key", "R")).isTrue();
        for (String id : c.ids) {
            assertThat(c.aliveSeenBy(id)).hasSize(8);
        }
    }

    @Test
    void shouldGossipWithAFewPeersPerRoundInsteadOfBroadcasting() {
        Cluster c = new Cluster(32, 12);
        c.nodes.get("n0").put("k", "v");

        c.net.run(3_000); // 30 rounds

        int max = 0;
        for (String id : c.ids) {
            max = Math.max(max, c.net.sentBy(id));
        }
        assertThat(max).as("a broadcast would be 31 messages per round (about 930)").isLessThan(250);
        assertThat(c.everyoneHas("k", "v")).isTrue();
    }

    @Test
    void shouldReplayTheSameRunFromTheSameSeed() {
        List<String> runs = new ArrayList<>();
        for (int i = 0; i < 2; i++) {
            Cluster c = new Cluster(10, 13);
            c.net.setDropRate(0.2);
            c.nodes.get("n0").put("k", "v");
            c.net.run(1_000);
            StringBuilder sb = new StringBuilder();
            for (String id : c.ids) {
                sb.append(c.nodes.get(id).get("k").isPresent() ? '1' : '0').append(c.net.sentBy(id)).append(',');
            }
            runs.add(sb.toString());
        }

        assertThat(runs.get(1)).isEqualTo(runs.get(0));
    }

    @Test
    void shouldConvergeOver100RandomSchedulesOfWritesAndLosses() {
        for (int seed = 0; seed < 100; seed++) {
            Cluster c = new Cluster(8, 500 + seed);
            java.util.Random r = new java.util.Random(seed);
            c.net.setDropRate(0.1 + r.nextInt(3) * 0.1);
            Set<String> written = new HashSet<>();
            for (int step = 0; step < 20; step++) {
                String key = "k" + r.nextInt(5);
                String value = "v" + step;
                c.nodes.get("n" + r.nextInt(8)).put(key, value);
                written.add(value);
                c.net.run(50 + r.nextInt(150));
            }
            c.net.setDropRate(0);

            c.net.run(3_000);

            for (int k = 0; k < 5; k++) {
                if (c.nodes.get("n0").get("k" + k).isPresent()) {
                    assertThat(c.agreeOn("k" + k)).as("seed %d key k%d", seed, k).isTrue();
                    assertThat(written).contains(c.nodes.get("n0").get("k" + k).get());
                }
            }
        }
    }
}
