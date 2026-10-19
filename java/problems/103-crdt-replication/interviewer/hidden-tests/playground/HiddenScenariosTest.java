package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Replica;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private static Replica replica(String id) {
        return Candidate.of(() -> Solution.replica(id));
    }

    @Test
    void shouldGiveTheSameResultForAnyMergeOrderOfTheSameSnapshots() {
        for (long seed = 0; seed < 50; seed++) {
            Random rnd = new Random(seed);
            List<String> snaps = new ArrayList<>();
            for (String id : List.of("a", "b", "c", "d")) {
                Replica r = replica(id);
                for (int i = 0; i < 20; i++) {
                    int ts = rnd.nextInt(10) * 100 + i;
                    switch (rnd.nextInt(5)) {
                        case 0 -> r.increment("c", rnd.nextInt(9));
                        case 1 -> r.decrement("c", rnd.nextInt(9));
                        case 2 -> r.add("s", "e" + rnd.nextInt(5));
                        case 3 -> r.remove("s", "e" + rnd.nextInt(5));
                        default -> r.write("r", "v" + rnd.nextInt(9), ts);
                    }
                }
                snaps.add(r.snapshot());
            }
            Replica first = replica("x");
            Replica second = replica("y");
            List<String> shuffled = new ArrayList<>(snaps);
            java.util.Collections.shuffle(shuffled, rnd);
            snaps.forEach(first::merge);
            shuffled.forEach(second::merge);
            shuffled.forEach(second::merge);

            assertThat(second.counter("c")).as("seed %d", seed).isEqualTo(first.counter("c"));
            assertThat(second.elements("s")).as("seed %d", seed).isEqualTo(first.elements("s"));
            assertThat(second.read("r")).as("seed %d", seed).isEqualTo(first.read("r"));
        }
    }

    @Test
    void shouldKeepSnapshotsSmallWhenNothingChanges() {
        Replica a = replica("a");
        for (int i = 0; i < 1_000; i++) a.increment("c", 1);
        int size = a.snapshot().length();
        for (int i = 0; i < 1_000; i++) a.increment("c", 1);

        assertThat(a.snapshot().length()).as("a counter must not grow with the number of increments").isLessThan(size * 2);
    }

    @Test
    void shouldScaleToManyElements() {
        Replica a = replica("a");
        Replica b = replica("b");
        for (int i = 0; i < 20_000; i++) a.add("s", "e" + i);
        for (int i = 0; i < 20_000; i += 2) a.remove("s", "e" + i);

        long start = System.nanoTime();
        b.merge(a.snapshot());

        assertThat(b.elements("s")).hasSize(10_000);
        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(5_000);
    }
}
