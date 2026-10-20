package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.HashRing;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final int KEYS = 50_000;

    private HashRing ring(int vnodes, int nodes) {
        HashRing r = Candidate.of(() -> Solution.create(vnodes));
        for (int i = 0; i < nodes; i++) {
            r.addNode("node-" + i);
        }
        return r;
    }

    private static Map<String, String> ownership(HashRing r) {
        Map<String, String> m = new HashMap<>();
        for (int k = 0; k < KEYS; k++) {
            m.put("key-" + k, r.nodeFor("key-" + k));
        }
        return m;
    }

    private static Map<String, Integer> counts(Map<String, String> ownership) {
        Map<String, Integer> c = new HashMap<>();
        ownership.values().forEach(n -> c.merge(n, 1, Integer::sum));
        return c;
    }

    @Test
    void shouldRefuseLookupsOnAnEmptyRing() {
        HashRing r = ring(100, 0);

        assertThatThrownBy(() -> r.nodeFor("k")).isInstanceOf(IllegalStateException.class);
        assertThat(r.nodesFor("k", 3)).isEmpty();
    }

    @Test
    void shouldSendEverythingToTheOnlyNode() {
        HashRing r = ring(100, 1);

        assertThat(counts(ownership(r))).containsOnlyKeys("node-0");
    }

    @Test
    void shouldRejectDuplicatesAndBadNodes() {
        HashRing r = ring(100, 2);

        assertThatThrownBy(() -> r.addNode("node-0")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> r.addNode(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> r.addNode("x", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(r.removeNode("nope")).isFalse();
    }

    @Test
    void shouldMapAKeyToTheSameNodeEveryTimeAndRegardlessOfInsertionOrder() {
        HashRing a = ring(100, 0);
        HashRing b = ring(100, 0);
        List<String> names = new ArrayList<>();
        for (int i = 0; i < 8; i++) {
            names.add("node-" + i);
        }
        names.forEach(a::addNode);
        List<String> shuffled = new ArrayList<>(names);
        Collections.shuffle(shuffled, new java.util.Random(3));
        shuffled.forEach(b::addNode);

        for (int k = 0; k < 5_000; k++) {
            assertThat(a.nodeFor("key-" + k)).isEqualTo(a.nodeFor("key-" + k)).isEqualTo(b.nodeFor("key-" + k));
        }
    }

    @Test
    void shouldSpreadKeysEvenly() {
        HashRing r = ring(200, 10);

        Map<String, Integer> counts = counts(ownership(r));

        assertThat(counts).hasSize(10);
        double mean = KEYS / 10.0;
        for (int c : counts.values()) {
            assertThat(c).isBetween((int) (mean * 0.65), (int) (mean * 1.35));
        }
    }

    @Test
    void shouldOnlyMoveKeysToTheNewNodeWhenOneJoins() {
        HashRing r = ring(200, 10);
        Map<String, String> before = ownership(r);

        r.addNode("newcomer");
        Map<String, String> after = ownership(r);

        int moved = 0;
        for (String key : before.keySet()) {
            if (!before.get(key).equals(after.get(key))) {
                moved++;
                assertThat(after.get(key)).as("%s moved between two old nodes", key).isEqualTo("newcomer");
            }
        }
        assertThat(moved / (double) KEYS).as("about 1/11 of the keys should move").isBetween(0.04, 0.15);
    }

    @Test
    void shouldOnlyMoveTheKeysOfTheNodeThatLeaves() {
        HashRing r = ring(200, 10);
        Map<String, String> before = ownership(r);

        assertThat(r.removeNode("node-3")).isTrue();
        Map<String, String> after = ownership(r);

        for (String key : before.keySet()) {
            if (before.get(key).equals("node-3")) {
                assertThat(after.get(key)).isNotEqualTo("node-3");
            } else {
                assertThat(after.get(key)).as("%s must not move", key).isEqualTo(before.get(key));
            }
        }
        assertThat(r.nodes()).hasSize(9).doesNotContain("node-3");
    }

    @Test
    void shouldReturnDistinctReplicaNodesStartingWithThePrimary() {
        HashRing r = ring(100, 6);

        for (int k = 0; k < 2_000; k++) {
            String key = "key-" + k;
            List<String> replicas = r.nodesFor(key, 3);
            assertThat(replicas).hasSize(3).doesNotHaveDuplicates();
            assertThat(replicas.get(0)).isEqualTo(r.nodeFor(key));
        }
        assertThat(r.nodesFor("key-1", 10)).hasSize(6).doesNotHaveDuplicates();
        assertThat(r.nodesFor("key-1", 1)).containsExactly(r.nodeFor("key-1"));
        assertThat(r.nodesFor("key-1", 0)).isEmpty();
    }

    @Test
    void shouldPromoteTheNextReplicaWhenThePrimaryLeaves() {
        HashRing r = ring(100, 6);
        Map<String, List<String>> before = new HashMap<>();
        for (int k = 0; k < 2_000; k++) {
            before.put("key-" + k, r.nodesFor("key-" + k, 3));
        }

        r.removeNode("node-2");

        for (Map.Entry<String, List<String>> e : before.entrySet()) {
            if (e.getValue().get(0).equals("node-2")) {
                assertThat(r.nodeFor(e.getKey())).isEqualTo(e.getValue().get(1));
            }
        }
    }

    @Test
    void shouldGiveHeavierNodesProportionallyMoreKeys() {
        HashRing r = ring(200, 0);
        r.addNode("small-1");
        r.addNode("small-2");
        r.addNode("big", 3);

        Map<String, Integer> counts = counts(ownership(r));

        double expectedBig = KEYS * 3 / 5.0;
        assertThat(counts.get("big")).isBetween((int) (expectedBig * 0.8), (int) (expectedBig * 1.2));
        assertThat(counts.get("small-1")).isBetween((int) (KEYS / 5.0 * 0.7), (int) (KEYS / 5.0 * 1.3));
    }

    @Test
    void shouldAnswerLookupsWhileNodesJoinAndLeave() {
        HashRing r = ring(100, 5);
        AtomicInteger bad = new AtomicInteger();
        Set<String> everSeen = new HashSet<>();
        for (int i = 0; i < 5; i++) {
            everSeen.add("node-" + i);
        }
        for (int i = 0; i < 20; i++) {
            everSeen.add("dyn-" + i);
        }

        Concurrently.run(8, t -> {
            if (t == 0) {
                for (int i = 0; i < 20; i++) {
                    r.addNode("dyn-" + i);
                    r.removeNode("dyn-" + (i / 2));
                }
                return;
            }
            for (int i = 0; i < 20_000; i++) {
                String owner = r.nodeFor("key-" + i);
                if (!everSeen.contains(owner) || r.nodesFor("key-" + i, 3).isEmpty()) {
                    bad.incrementAndGet();
                }
            }
        });

        assertThat(bad).hasValue(0);
        assertThat(r.nodes()).contains("node-0", "node-4");
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            HashRing r = ring(50, 4);

            Concurrently.run(4, t -> {
                r.addNode("extra-" + t);
                for (int i = 0; i < 100; i++) {
                    r.nodeFor("k" + i);
                }
            });

            assertThat(r.nodes()).as("run %d", run).hasSize(8);
            assertThat(r.nodesFor("k", 8)).as("run %d", run).hasSize(8).doesNotHaveDuplicates();
        }
    }
}
