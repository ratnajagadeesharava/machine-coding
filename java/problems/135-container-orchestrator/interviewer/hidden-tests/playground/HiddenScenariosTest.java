package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Orchestrator;
import contract.Strategy;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static Orchestrator orch(Strategy s) {
        return Candidate.of(() -> Solution.create(s));
    }

    @Test
    void shouldReplaceAnEvictedContainerOnAnotherNodeImmediately() {
        Orchestrator o = orch(Strategy.FIRST_FIT);
        o.addNode("n1", 1000, 1000);
        o.addNode("n2", 400, 400);
        o.submit("v", 400, 400, 1);
        o.submit("u", 100, 100, 300);

        assertThat(o.submit("hi", 700, 700, 100)).contains("n1");

        assertThat(o.placement("v")).isEqualTo("n2");
        assertThat(o.pending()).isEmpty();
        assertThat(o.nodeUsage("n1")).isEqualTo("800/1000,800/1000");
    }

    @Test
    void shouldNeverPreemptWhenRetryingPendingContainers() {
        Orchestrator o = orch(Strategy.FIRST_FIT);
        o.addNode("n1", 100, 100);
        o.addNode("n2", 100, 100);
        o.submit("important", 100, 100, 150); // n1
        o.submit("low", 100, 100, 1); // n2

        assertThat(o.removeNode("n1")).containsExactly("important->PENDING");

        assertThat(o.placement("low")).isEqualTo("n2");
        assertThat(o.pending()).containsExactly("important");
        o.addNode("n3", 100, 100);
        assertThat(o.placement("important")).isEqualTo("n3");
        assertThat(o.placement("low")).isEqualTo("n2");
    }

    @Test
    void shouldKeepTheOriginalSubmissionOrderOfContainersDisplacedByNodeRemoval() {
        Orchestrator o = orch(Strategy.FIRST_FIT);
        o.addNode("n1", 100, 100);
        o.addNode("n2", 100, 100);
        o.submit("a", 100, 100, 1); // n1, oldest
        o.submit("c", 100, 100, 1); // n2
        o.submit("b", 100, 100, 1); // pending, newer than a

        o.removeNode("n1");
        assertThat(o.pending()).containsExactly("a", "b");

        o.addNode("n3", 100, 100);
        assertThat(o.placement("a")).isEqualTo("n3");
        assertThat(o.pending()).containsExactly("b");
    }

    @Test
    void shouldChooseVictimsByLowestPriorityThenNewestAcrossPriorities() {
        for (int needed : new int[] {400, 700}) {
            Orchestrator o = orch(Strategy.FIRST_FIT);
            o.addNode("n", 1000, 1000);
            o.submit("a", 300, 300, 3);
            o.submit("b", 300, 300, 1);
            o.submit("c", 300, 300, 3);

            assertThat(o.submit("hi", needed, needed, 100)).contains("n");

            if (needed == 400) {
                assertThat(o.containersOn("n")).containsExactly("a", "c", "hi");
            } else {
                assertThat(o.containersOn("n")).containsExactly("a", "hi"); // b, then the newer of the prio-3 pair
            }
        }
    }

    @Test
    void shouldLetAHigherPreemptorEvictContainersThatThemselvesCouldPreempt() {
        Orchestrator o = orch(Strategy.FIRST_FIT);
        o.addNode("n", 100, 100);
        o.submit("r", 100, 100, 100);

        assertThat(o.submit("s", 100, 100, 200)).contains("n");
        assertThat(o.placement("r")).isEqualTo("PENDING");
        assertThat(o.submit("t", 100, 100, 150)).isEmpty(); // 150 < 200: cannot evict s
    }

    @Test
    void shouldChoosePreemptionNodeIndependentlyOfTheStrategy() {
        Orchestrator o = orch(Strategy.WORST_FIT);
        o.addNode("n1", 1000, 1000);
        o.addNode("n2", 2000, 2000);
        o.submit("y", 1900, 1900, 1); // only n2 can host it
        o.submit("x", 900, 900, 1); //   n1 (n2 has 100 left)

        // both nodes would need one eviction; the tie goes to n1 although worst fit prefers the bigger n2
        assertThat(o.submit("urgent", 950, 950, 100)).contains("n1");
        assertThat(o.placement("x")).isEqualTo("PENDING");
        assertThat(o.placement("y")).isEqualTo("n2");
    }

    @Test
    void shouldKeepInvariantsThroughALongMixedSequence() {
        Random rnd = new Random(12345);
        for (Strategy s : Strategy.values()) {
            Orchestrator o = orch(s);
            Model m = new Model(o);
            for (int i = 0; i < 400; i++) {
                int op = rnd.nextInt(100);
                if (op < 8 || m.caps.size() < 2) {
                    m.addNode("n" + rnd.nextInt(12), 200 + rnd.nextInt(800), 200 + rnd.nextInt(800));
                } else if (op < 14) {
                    m.removeNode("n" + rnd.nextInt(12));
                } else if (op < 22) {
                    o.setStrategy(Strategy.values()[rnd.nextInt(3)]);
                } else if (op < 72) {
                    int prio = rnd.nextInt(4) == 0 ? 100 + rnd.nextInt(3) * 50 : rnd.nextInt(60);
                    m.submit("c" + i, 20 + rnd.nextInt(300), 20 + rnd.nextInt(300), prio);
                } else {
                    m.removeRandom(rnd);
                }
                m.check();
            }
        }
    }

    @Test
    void shouldStayConsistentUnderConcurrentSubmitsRemovalsAndNodeChurn() {
        Orchestrator o = orch(Strategy.BEST_FIT);
        Model m = new Model(o);
        for (int i = 0; i < 6; i++) {
            m.addNode("base" + i, 1000, 1000);
        }
        Map<String, int[]> sizes = new java.util.concurrent.ConcurrentHashMap<>();

        Concurrently.run(16, 60_000, t -> {
            if (t >= 12) {
                for (int i = 0; i < 300; i++) {
                    String id = "extra" + (t - 12);
                    try {
                        if (i % 2 == 0) {
                            o.addNode(id, 600, 600);
                        } else {
                            o.removeNode(id);
                        }
                    } catch (IllegalArgumentException ignored) {
                        // never happens: each churn thread owns its node id
                    }
                }
                return;
            }
            for (int i = 0; i < 150; i++) {
                String id = "t" + t + "-" + i;
                int cpu = 10 * (1 + i % 5);
                int mem = 10 * (1 + (i / 5) % 4);
                sizes.put(id, new int[] {cpu, mem});
                o.submit(id, cpu, mem, (i % 7 == 0) ? 100 : i % 5);
                if (i % 3 == 2) {
                    o.remove("t" + t + "-" + (i - 2));
                    sizes.remove("t" + t + "-" + (i - 2));
                }
            }
        });

        // final state: usage = sum of sizes, within capacity, every live container in exactly one place
        Set<String> seen = new HashSet<>();
        List<String> nodeIds = new ArrayList<>();
        for (int i = 0; i < 6; i++) {
            nodeIds.add("base" + i);
        }
        for (int i = 0; i < 4; i++) {
            nodeIds.add("extra" + i);
        }
        for (String n : nodeIds) {
            String usage;
            List<String> on;
            try {
                usage = o.nodeUsage(n);
                on = o.containersOn(n);
            } catch (IllegalArgumentException gone) {
                continue;
            }
            long cpu = 0;
            long mem = 0;
            for (String c : on) {
                assertThat(seen.add(c)).as("container %s listed twice", c).isTrue();
                cpu += sizes.get(c)[0];
                mem += sizes.get(c)[1];
            }
            String[] p = usage.split("[/,]");
            assertThat(Long.parseLong(p[0])).isEqualTo(cpu).isLessThanOrEqualTo(Long.parseLong(p[1]));
            assertThat(Long.parseLong(p[2])).isEqualTo(mem).isLessThanOrEqualTo(Long.parseLong(p[3]));
        }
        for (String c : o.pending()) {
            assertThat(seen.add(c)).as("container %s both pending and running or listed twice", c).isTrue();
        }
        assertThat(seen).isEqualTo(sizes.keySet());
    }

    /** Sequential bookkeeping next to the system under test, checking structural invariants after each step. */
    private static final class Model {
        final Orchestrator o;
        final Map<String, long[]> caps = new HashMap<>();
        final Map<String, long[]> live = new HashMap<>(); // id -> cpu, mem, prio, seq
        long seq;

        Model(Orchestrator o) {
            this.o = o;
        }

        void addNode(String id, int cpu, int mem) {
            if (caps.containsKey(id)) {
                return;
            }
            o.addNode(id, cpu, mem);
            caps.put(id, new long[] {cpu, mem});
        }

        void removeNode(String id) {
            if (!caps.containsKey(id)) {
                return;
            }
            List<String> on = o.containersOn(id);
            List<String> result = o.removeNode(id);
            caps.remove(id);
            assertThat(result).hasSize(on.size());
            for (int i = 0; i < on.size(); i++) {
                assertThat(result.get(i)).startsWith(on.get(i) + "->");
            }
        }

        void submit(String id, int cpu, int mem, int prio) {
            live.put(id, new long[] {cpu, mem, prio, seq++});
            o.submit(id, cpu, mem, prio);
        }

        void removeRandom(Random rnd) {
            if (live.isEmpty()) {
                return;
            }
            List<String> ids = new ArrayList<>(live.keySet());
            ids.sort(null);
            String id = ids.get(rnd.nextInt(ids.size()));
            o.remove(id);
            live.remove(id);
        }

        void check() {
            Set<String> seen = new HashSet<>();
            Map<String, long[]> free = new HashMap<>();
            for (Map.Entry<String, long[]> e : caps.entrySet()) {
                long cpu = 0;
                long mem = 0;
                List<String> on = o.containersOn(e.getKey());
                assertThat(on).isSorted();
                for (String c : on) {
                    assertThat(seen.add(c)).isTrue();
                    cpu += live.get(c)[0];
                    mem += live.get(c)[1];
                    assertThat(o.placement(c)).isEqualTo(e.getKey());
                }
                assertThat(cpu).isLessThanOrEqualTo(e.getValue()[0]);
                assertThat(mem).isLessThanOrEqualTo(e.getValue()[1]);
                assertThat(o.nodeUsage(e.getKey()))
                        .isEqualTo(cpu + "/" + e.getValue()[0] + "," + mem + "/" + e.getValue()[1]);
                free.put(e.getKey(), new long[] {e.getValue()[0] - cpu, e.getValue()[1] - mem});
            }
            List<String> pend = o.pending();
            long[] prev = null;
            for (String c : pend) {
                assertThat(seen.add(c)).isTrue();
                assertThat(o.placement(c)).isEqualTo("PENDING");
                long[] cur = live.get(c);
                if (prev != null) {
                    assertThat(prev[2] > cur[2] || (prev[2] == cur[2] && prev[3] < cur[3]))
                            .as("pending order")
                            .isTrue();
                }
                prev = cur;
                for (long[] f : free.values()) {
                    assertThat(cur[0] <= f[0] && cur[1] <= f[1])
                            .as("pending container %s fits a node but was not placed", c)
                            .isFalse();
                }
            }
            assertThat(seen).isEqualTo(live.keySet());
        }
    }
}
