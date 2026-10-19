package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Gateway;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldNotSerializeSlowBackendCalls() {
        FakeClock clock = new FakeClock(0);
        Gateway g = Candidate.of(() -> Solution.create(clock));
        g.registerBackend("slow", path -> {
            Thread.sleep(50);
            return "ok";
        });
        g.addRoute("/slow", List.of("slow"));
        g.addClient("k", "c", 100_000);

        long start = System.nanoTime();
        Concurrently.run(32, 60_000, t -> {
            for (int i = 0; i < 5; i++) {
                g.handle("k", "/slow/x");
            }
        });
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).as("160 calls of 50 ms: serialised would take 8 s").isLessThan(3_000);
    }

    @Test
    void shouldOpenTheCircuitWithinAFewCallsEvenWhenManyThreadsHitAFailingBackend() {
        FakeClock clock = new FakeClock(0);
        Gateway g = Candidate.of(() -> Solution.create(clock));
        AtomicInteger calls = new AtomicInteger();
        g.registerBackend("bad", path -> {
            calls.incrementAndGet();
            throw new IllegalStateException("down");
        });
        g.addRoute("/bad", List.of("bad"));
        g.addClient("k", "c", 100_000);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 100; i++) {
                g.handle("k", "/bad/x");
            }
        });

        assertThat(calls.get()).as("at most 3 + the requests already in flight").isLessThanOrEqualTo(3 + 16);
    }
}
