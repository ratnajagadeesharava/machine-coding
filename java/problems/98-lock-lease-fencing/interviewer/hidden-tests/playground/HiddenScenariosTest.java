package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LockService;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldHandleAHundredThousandIndependentLocks() {
        FakeClock clock = new FakeClock(0);
        LockService l = Candidate.of(() -> Solution.create(clock));

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 12_500; i++) {
                l.acquire("lock-" + (t * 12_500 + i), "o", 1_000);
            }
        });

        assertThat(l.latestToken("lock-99999")).isEqualTo(1);
        assertThat(l.holder("lock-5")).hasValue("o");
    }

    @Test
    void shouldKeepTokensStrictlyIncreasingWhenLeasesExpireAndAreStolenConstantly() {
        FakeClock clock = new FakeClock(0);
        LockService l = Candidate.of(() -> Solution.create(clock));
        AtomicLong regressions = new AtomicLong();

        Concurrently.run(8, 60_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 1_000; i++) {
                    clock.advance(10); // the previous 5 ms lease has expired
                    long token = l.acquire("hot", "owner-" + i, 5).orElseThrow();
                    if (token != i + 1) {
                        regressions.incrementAndGet();
                    }
                }
                return;
            }
            long last = 0;
            for (int i = 0; i < 20_000; i++) {
                long now = l.latestToken("hot");
                if (now < last) {
                    regressions.incrementAndGet();
                }
                last = now;
                l.holder("hot");
            }
        });

        assertThat(regressions).hasValue(0);
        assertThat(l.latestToken("hot")).isEqualTo(1_000);
    }
}
