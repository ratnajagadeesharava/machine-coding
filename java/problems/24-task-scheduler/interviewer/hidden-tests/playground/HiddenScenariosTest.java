package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.TaskSchedulerApi;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private final FakeClock clock = new FakeClock(5_000);
    private final List<String> log = new ArrayList<>();

    private TaskSchedulerApi scheduler() {
        return Candidate.of(() -> Solution.create(clock));
    }

    @Test
    void sameDueTimeShouldFollowSubmissionOrderAcrossTaskKinds() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(20, () -> log.add("once-1"));
        s.scheduleRecurring(20, 100, () -> log.add("rec"));
        s.scheduleOnce(20, () -> log.add("once-2"));

        clock.advance(20);

        assertThat(s.runDue()).isEqualTo(3);
        assertThat(log).containsExactly("once-1", "rec", "once-2");
    }

    @Test
    void shouldCatchUpAThousandOccurrencesOfAOneMillisecondPeriod() {
        TaskSchedulerApi s = scheduler();
        AtomicInteger runs = new AtomicInteger();
        s.scheduleRecurring(0, 1, runs::incrementAndGet);   // due at 5000, 5001, ...

        clock.advance(999);                                  // now 5999 -> occurrences 5000..5999

        assertThat(s.runDue()).isEqualTo(1_000);
        assertThat(runs.get()).isEqualTo(1_000);
    }

    @Test
    void aOneShotCancellingItselfShouldBeRefusedBecauseItAlreadyStarted() {
        TaskSchedulerApi s = scheduler();
        AtomicReference<String> self = new AtomicReference<>();
        AtomicBoolean result = new AtomicBoolean(true);
        self.set(s.scheduleOnce(0, () -> result.set(s.cancel(self.get()))));

        assertThat(s.runDue()).isEqualTo(1);

        assertThat(result).isFalse();
    }

    @Test
    void callingRunDueTwiceInARowShouldRunNothingTheSecondTime() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(0, () -> log.add("a"));
        s.scheduleRecurring(0, 50, () -> log.add("r"));

        assertThat(s.runDue()).isEqualTo(2);
        assertThat(s.runDue()).isZero();
        assertThat(log).containsExactly("a", "r");
    }

    @Test
    void aThrowingOneShotShouldStillCountAsAnExecutionAndNotRunAgain() {
        TaskSchedulerApi s = scheduler();
        AtomicInteger attempts = new AtomicInteger();
        s.scheduleOnce(0, () -> {
            attempts.incrementAndGet();
            throw new RuntimeException("boom");
        });

        assertThat(s.runDue()).isEqualTo(1);
        clock.advance(1_000);
        assertThat(s.runDue()).isZero();
        assertThat(attempts.get()).isEqualTo(1);
    }
}
