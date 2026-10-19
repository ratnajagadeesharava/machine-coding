package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SharedCache;
import java.util.concurrent.ThreadLocalRandom;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldServeAReadHeavyWorkloadWithAFewWriters() {
        SharedCache<Integer, String> c = Candidate.of(() -> Solution.<Integer, String>create(1_000));
        for (int k = 0; k < 1_000; k++) {
            c.put(k, "v" + k);
        }
        AtomicLong hits = new AtomicLong();

        Concurrently.run(16, 60_000, t -> {
            ThreadLocalRandom rnd = ThreadLocalRandom.current();
            for (int i = 0; i < 200_000; i++) {
                int key = rnd.nextInt(1_500);
                if (t == 0 && i % 50 == 0) {
                    c.put(key, "v" + key);
                } else if (c.peek(key).isPresent()) {
                    hits.incrementAndGet();
                }
            }
        });

        assertThat(hits.get()).isPositive();
        assertThat(c.size()).isLessThanOrEqualTo(1_000);
    }

    @Test
    void shouldKeepWorkingWhenEveryOperationFightsOverOneKey() {
        SharedCache<Integer, String> c = Candidate.of(() -> Solution.<Integer, String>create(3));

        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 50_000; i++) {
                c.put(7, "v" + t);
                c.get(7);
                if (i % 7 == 0) {
                    c.remove(7);
                }
            }
        });

        assertThat(c.size()).isLessThanOrEqualTo(1);
    }
}
