package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.JobService;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldHandleAThousandJobsOverADay() {
        FakeClock clock = new FakeClock(0);
        JobService s = Candidate.of(() -> Solution.create(clock));
        AtomicInteger runs = new AtomicInteger();
        for (int i = 0; i < 1_000; i++) {
            s.define("job" + i, "*/" + (1 + i % 60) + " * *", runs::incrementAndGet, 1, 1_000, 1_000);
        }

        for (int minute = 1; minute <= 1_440; minute++) {
            clock.advance(60_000);
            s.runDue();
        }

        assertThat(runs.get()).isGreaterThan(20_000);
    }

    @Test
    void shouldJumpOverWeeksOfDowntimeWithOneRunPerJob() {
        FakeClock clock = new FakeClock(0);
        JobService s = Candidate.of(() -> Solution.create(clock));
        AtomicInteger runs = new AtomicInteger();
        s.define("j", "* * *", runs::incrementAndGet, 1, 1_000, 1_000);

        clock.advance(21L * 1_440 * 60_000);

        assertThat(s.runDue()).isEqualTo(1);
        assertThat(runs).hasValue(1);
    }
}
