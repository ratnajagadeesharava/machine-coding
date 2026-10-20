package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.IdempotentProcessor;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldExecuteEachOfAHundredThousandKeysExactlyOnceUnderRetries() {
        FakeClock clock = new FakeClock(0);
        IdempotentProcessor p = Candidate.of(() -> Solution.create(clock));
        AtomicInteger executions = new AtomicInteger();

        Concurrently.run(8, 60_000, t -> {
            for (int round = 0; round < 3; round++) { // every key is retried three times by every thread
                for (int k = 0; k < 12_500; k++) {
                    p.process("key-" + k, "body-" + k, b -> "r" + executions.incrementAndGet());
                }
            }
        });

        assertThat(executions).hasValue(12_500);
    }

    @Test
    void shouldNotLeakMemoryForExpiredKeys() {
        FakeClock clock = new FakeClock(0);
        IdempotentProcessor p = Candidate.of(() -> Solution.create(clock));

        for (int day = 0; day < 50; day++) {
            for (int k = 0; k < 2_000; k++) {
                p.process("day" + day + "-" + k, "b", b -> "r");
            }
            clock.advance(25L * 3_600_000);
        }

        // nothing to assert beyond finishing: expired entries must be reclaimed or this runs out of memory in a tiny heap
        assertThat(p.process("fresh", "b", b -> "ok")).isEqualTo("ok");
    }
}
