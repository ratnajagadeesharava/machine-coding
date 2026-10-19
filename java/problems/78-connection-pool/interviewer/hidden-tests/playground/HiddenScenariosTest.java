package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Pool;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandleInterruptedWaitersWithoutLosingAConnection() throws Exception {
        Pool p = Candidate.of(() -> Solution.create(1));
        int id = p.acquire(0).orElseThrow();
        AtomicInteger interrupted = new AtomicInteger();
        Thread[] waiters = new Thread[10];
        for (int i = 0; i < waiters.length; i++) {
            waiters[i] = new Thread(() -> {
                try {
                    p.acquire(30_000);
                } catch (InterruptedException e) {
                    interrupted.incrementAndGet();
                }
            });
            waiters[i].setDaemon(true);
            waiters[i].start();
        }
        Thread.sleep(100);

        for (Thread w : waiters) {
            w.interrupt();
        }
        for (Thread w : waiters) {
            w.join(5_000);
        }
        p.release(id);

        assertThat(interrupted).hasValue(10);
        assertThat(p.available()).isEqualTo(1);
        assertThat(p.acquire(0)).isPresent();
    }

    @Test
    void shouldSustainFiftyThousandLeasesOnASmallPool() {
        Pool p = Candidate.of(() -> Solution.create(5));
        AtomicInteger leases = new AtomicInteger();

        Concurrently.run(32, 60_000, t -> {
            for (int i = 0; i < 1_600; i++) {
                try {
                    Optional<Integer> id = p.acquire(30_000);
                    leases.incrementAndGet();
                    p.release(id.orElseThrow());
                } catch (InterruptedException e) {
                    throw new AssertionError(e);
                }
            }
        });

        assertThat(leases).hasValue(32 * 1_600);
        assertThat(p.available()).isEqualTo(5);
    }
}
