package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Job;
import contract.JobService;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long MIN = 60_000;
    private final FakeClock clock = new FakeClock(0);

    private JobService jobs() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private void minutes(long m) {
        clock.advance(m * MIN);
    }

    private static Job counting(AtomicInteger runs) {
        return runs::incrementAndGet;
    }

    /** Fails the first {@code failures} calls, then succeeds. */
    private static Job flaky(int failures, AtomicInteger calls) {
        return () -> {
            if (calls.incrementAndGet() <= failures) {
                throw new IllegalStateException("boom " + calls.get());
            }
        };
    }

    @Test
    void shouldRunAnEveryFiveMinutesJobOnTime() {
        JobService s = jobs();
        AtomicInteger runs = new AtomicInteger();
        s.define("j", "*/5 * *", counting(runs), 1, 1_000, 1_000);

        assertThat(s.runDue()).isZero();
        minutes(4);
        assertThat(s.runDue()).isZero();
        minutes(1);
        assertThat(s.runDue()).isEqualTo(1);
        assertThat(s.runDue()).isZero(); // nothing twice
        minutes(5);
        assertThat(s.runDue()).isEqualTo(1);

        assertThat(runs).hasValue(2);
        assertThat(s.history("j")).containsExactly("5:1:OK", "10:1:OK");
        assertThat(s.nextSlot("j")).hasValue(15L);
    }

    @Test
    void shouldNotRunInTheMinuteOfTheDefinition() {
        JobService s = jobs();
        s.define("j", "* * *", () -> {}, 1, 1_000, 1_000);

        assertThat(s.nextSlot("j")).hasValue(1L);
        assertThat(s.runDue()).isZero();
    }

    @Test
    void shouldRunADailyJobAtTheRightMinuteOfTheDay() {
        JobService s = jobs();
        s.define("daily", "30 9 *", () -> {}, 1, 1_000, 1_000);

        assertThat(s.nextSlot("daily")).hasValue(9L * 60 + 30);
        minutes(9 * 60 + 30);
        s.runDue();
        assertThat(s.nextSlot("daily")).hasValue(1_440L + 9 * 60 + 30);
    }

    @Test
    void shouldUseTheWeekdayFieldWithSundayAsZero() {
        JobService s = jobs();
        s.define("monday", "0 12 1", () -> {}, 1, 1_000, 1_000); // epoch day 0 is a Thursday
        s.define("sunday", "0 0 0", () -> {}, 1, 1_000, 1_000);

        assertThat(s.nextSlot("monday")).hasValue(4L * 1_440 + 12 * 60); // Mon 1970-01-05 12:00
        assertThat(s.nextSlot("sunday")).hasValue(3L * 1_440); // Sun 1970-01-04 00:00
    }

    @Test
    void shouldUnderstandListsAndRanges() {
        JobService s = jobs();
        s.define("j", "0,30 8-9 *", () -> {}, 1, 1_000, 1_000);

        assertThat(s.nextSlot("j")).hasValue(480L); // 08:00
        minutes(480);
        s.runDue();
        assertThat(s.nextSlot("j")).hasValue(510L); // 08:30
        minutes(30);
        s.runDue();
        assertThat(s.nextSlot("j")).hasValue(540L); // 09:00
        minutes(30);
        s.runDue();
        assertThat(s.nextSlot("j")).hasValue(570L); // 09:30
        minutes(30);
        s.runDue();

        assertThat(s.history("j")).containsExactly("480:1:OK", "510:1:OK", "540:1:OK", "570:1:OK");
        assertThat(s.nextSlot("j")).hasValue(1_440L + 480); // 10:00 is outside 8-9, so it wraps to the next day
    }

    @Test
    void shouldRejectBadDefinitions() {
        JobService s = jobs();
        s.define("ok", "* * *", () -> {}, 1, 1, 1);

        assertThatThrownBy(() -> s.define("ok", "* * *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("a", "* *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("b", "60 * *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("c", "* 24 *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("d", "* * 7", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("e", "*/0 * *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("f", "5-1 * *", () -> {}, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("g", "* * *", () -> {}, 0, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.define("h", "* * *", () -> {}, 1, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.nextSlot("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRunOnlyOnceForManyMissedSlotsAndForgetTheOthers() {
        JobService s = jobs();
        AtomicInteger runs = new AtomicInteger();
        s.define("j", "*/5 * *", counting(runs), 1, 1_000, 1_000);

        minutes(17);

        assertThat(s.runDue()).isEqualTo(1);
        assertThat(runs).hasValue(1);
        assertThat(s.history("j")).containsExactly("15:1:OK"); // latest matching minute <= 17
        assertThat(s.nextSlot("j")).hasValue(20L);
    }

    @Test
    void shouldRetryWithExponentialBackoffUntilItSucceeds() {
        JobService s = jobs();
        AtomicInteger calls = new AtomicInteger();
        s.define("j", "*/10 * *", flaky(2, calls), 5, 1_000, 60_000);
        minutes(10);

        assertThat(s.runDue()).isEqualTo(1); // attempt 1 fails
        clock.advance(999);
        assertThat(s.runDue()).isZero(); // retry due 1s after the failure
        clock.advance(1);
        assertThat(s.runDue()).isEqualTo(1); // attempt 2 fails
        clock.advance(1_999);
        assertThat(s.runDue()).isZero(); // second backoff is 2s
        clock.advance(1);
        assertThat(s.runDue()).isEqualTo(1); // attempt 3 succeeds

        assertThat(s.history("j")).containsExactly("10:1:FAILED", "10:2:FAILED", "10:3:OK");
        assertThat(s.deadLetters()).isEmpty();
        assertThat(s.nextSlot("j")).hasValue(20L);
    }

    @Test
    void shouldCapTheBackoff() {
        JobService s = jobs();
        s.define("j", "*/60 * *", () -> { throw new RuntimeException("always"); }, 6, 1_000, 3_000);
        minutes(60);
        long[] expectedGaps = {1_000, 2_000, 3_000, 3_000, 3_000};

        s.runDue();
        for (long gap : expectedGaps) {
            clock.advance(gap - 1);
            assertThat(s.runDue()).as("too early for gap %d", gap).isZero();
            clock.advance(1);
            assertThat(s.runDue()).isEqualTo(1);
        }

        assertThat(s.history("j")).hasSize(6).last().isEqualTo("60:6:DEAD");
    }

    @Test
    void shouldGiveUpAfterTheLastAttemptAndKeepSchedulingTheNextSlot() {
        JobService s = jobs();
        AtomicInteger calls = new AtomicInteger();
        s.define("j", "*/10 * *", flaky(3, calls), 3, 1_000, 1_000);
        minutes(10);

        s.runDue();
        clock.advance(1_000);
        s.runDue();
        clock.advance(1_000);
        s.runDue();

        assertThat(s.history("j")).containsExactly("10:1:FAILED", "10:2:FAILED", "10:3:DEAD");
        assertThat(s.deadLetters()).containsExactly("j:10");
        clock.advance(10 * MIN - 2_000);
        assertThat(s.runDue()).isEqualTo(1); // the next slot runs again (the job works now)
        assertThat(s.history("j")).last().isEqualTo("20:1:OK");
    }

    @Test
    void shouldSkipASlotThatArrivesWhileTheJobIsStillRetrying() {
        JobService s = jobs();
        AtomicInteger calls = new AtomicInteger();
        s.define("j", "*/5 * *", flaky(1, calls), 3, 8 * MIN, 8 * MIN);
        minutes(5);
        s.runDue(); // slot 5 fails, retry due at minute 13

        minutes(5); // slot 10 arrives
        assertThat(s.runDue()).isZero();
        minutes(3); // minute 13: the retry runs
        assertThat(s.runDue()).isEqualTo(1);

        assertThat(s.history("j")).containsExactly("5:1:FAILED", "10:0:SKIPPED", "5:2:OK");
        assertThat(s.nextSlot("j")).hasValue(15L);
    }

    @Test
    void shouldRunDueJobsInDueTimeThenDefinitionOrder() {
        JobService s = jobs();
        List<String> order = new java.util.concurrent.CopyOnWriteArrayList<>();
        s.define("b", "*/10 * *", () -> order.add("b"), 1, 1, 1);
        s.define("a", "*/5 * *", () -> order.add("a"), 1, 1, 1);
        s.define("c", "*/10 * *", () -> order.add("c"), 1, 1, 1);

        minutes(10);
        s.runDue();

        assertThat(order).containsExactly("a", "b", "c"); // a is due since minute 5; b and c since 10, in definition order
    }

    @Test
    void shouldNotLetAFailingJobStopTheOthers() {
        JobService s = jobs();
        AtomicInteger runs = new AtomicInteger();
        s.define("bad", "* * *", () -> { throw new Exception("x"); }, 1, 1_000, 1_000);
        s.define("good", "* * *", counting(runs), 1, 1_000, 1_000);
        minutes(1);

        assertThat(s.runDue()).isEqualTo(2);
        assertThat(runs).hasValue(1);
    }

    @Test
    void shouldPauseResumeAndRemove() {
        JobService s = jobs();
        AtomicInteger runs = new AtomicInteger();
        s.define("j", "* * *", counting(runs), 1, 1_000, 1_000);

        s.pause("j");
        assertThat(s.nextSlot("j")).isEmpty();
        minutes(5);
        assertThat(s.runDue()).isZero();
        s.resume("j");
        assertThat(s.nextSlot("j")).hasValue(6L);
        minutes(1);
        assertThat(s.runDue()).isEqualTo(1);
        s.remove("j");
        minutes(1);
        assertThat(s.runDue()).isZero();
        assertThatThrownBy(() -> s.history("j")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldExecuteEachAttemptExactlyOnceWhenManyThreadsCallRunDue() {
        JobService s = jobs();
        AtomicInteger runs = new AtomicInteger();
        s.define("j", "* * *", counting(runs), 1, 1_000, 1_000);
        AtomicInteger executions = new AtomicInteger();

        for (int tick = 0; tick < 50; tick++) {
            minutes(1);
            Concurrently.run(8, t -> executions.addAndGet(s.runDue()));
        }

        assertThat(runs).hasValue(50);
        assertThat(executions).hasValue(50);
        assertThat(s.history("j")).hasSize(50);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            FakeClock c = new FakeClock(0);
            JobService s = Candidate.of(() -> Solution.create(c));
            AtomicInteger calls = new AtomicInteger();
            s.define("j", "* * *", flaky(1, calls), 3, 1_000, 1_000);
            c.advance(MIN);

            Concurrently.run(6, t -> s.runDue());
            c.advance(1_000);
            Concurrently.run(6, t -> s.runDue());

            assertThat(calls).as("run %d", run).hasValue(2);
            assertThat(s.history("j")).as("run %d", run).containsExactly("1:1:FAILED", "1:2:OK");
        }
    }
}
