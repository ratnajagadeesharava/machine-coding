package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.HashRing;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldLookUpAMillionKeysOnALargeRingQuickly() {
        HashRing r = Candidate.of(() -> Solution.create(200));
        for (int i = 0; i < 1_000; i++) {
            r.addNode("node-" + i);
        }

        long start = System.nanoTime();
        for (int k = 0; k < 1_000_000; k++) {
            r.nodeFor("key-" + k);
        }

        assertThat((System.nanoTime() - start) / 1_000_000).as("ms").isLessThan(8_000);
    }

    @Test
    void shouldKeepTheBalanceWhenManyNodesJoinAndLeave() {
        HashRing r = Candidate.of(() -> Solution.create(200));
        for (int i = 0; i < 50; i++) {
            r.addNode("n" + i);
        }
        for (int i = 0; i < 25; i++) {
            r.removeNode("n" + i);
        }
        for (int i = 50; i < 75; i++) {
            r.addNode("n" + i);
        }

        java.util.Map<String, Integer> counts = new java.util.HashMap<>();
        for (int k = 0; k < 100_000; k++) {
            counts.merge(r.nodeFor("key-" + k), 1, Integer::sum);
        }

        assertThat(counts).hasSize(50);
        assertThat(counts.values().stream().mapToInt(Integer::intValue).max().getAsInt()).isLessThan(2 * 100_000 / 50 * 3 / 2);
    }
}
