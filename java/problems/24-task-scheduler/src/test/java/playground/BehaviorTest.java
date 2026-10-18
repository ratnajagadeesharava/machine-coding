package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

import contract.TaskSchedulerApi;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicIntegerArray;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {
    private static final long T0 = 1_000;

    private final FakeClock clock = new FakeClock(T0);
    private final List<String> log = new ArrayList<>();

    private TaskSchedulerApi scheduler() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private Runnable note(String label) {
        return () -> log.add(label);
    }

    @Test
    void shouldNotRunTaskBeforeItsDelay() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(100, note("a"));

        assertThat(s.runDue()).isZero();
        clock.advance(99);
        assertThat(s.runDue()).isZero();
        clock.advance(1);
        assertThat(s.runDue()).isEqualTo(1);

        assertThat(log).containsExactly("a");
    }

    @Test
    void shouldRunZeroDelayTaskImmediately() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(0, note("a"));

        assertThat(s.runDue()).isEqualTo(1);
        assertThat(log).containsExactly("a");
    }

    @Test
    void shouldRunOneShotTaskOnlyOnce() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(10, note("a"));

        clock.advance(10);
        assertThat(s.runDue()).isEqualTo(1);
        clock.advance(1_000);
        assertThat(s.runDue()).isZero();

        assertThat(log).containsExactly("a");
    }

    @Test
    void shouldRunDueTasksInDueTimeThenSubmissionOrder() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(50, note("a"));
        s.scheduleOnce(10, note("b"));
        s.scheduleOnce(50, note("c"));
        s.scheduleOnce(10, note("d"));
        s.scheduleOnce(500, note("late"));

        clock.advance(50);

        assertThat(s.runDue()).isEqualTo(4);
        assertThat(log).containsExactly("b", "d", "a", "c");
    }

    @Test
    void shouldReturnUniqueIds() {
        TaskSchedulerApi s = scheduler();
        String a = s.scheduleOnce(10, note("a"));
        String b = s.scheduleOnce(10, note("b"));
        String c = s.scheduleRecurring(10, 10, note("c")).orElseThrow();

        assertThat(new HashSet<>(List.of(a, b, c))).hasSize(3);
    }

    @Test
    void shouldRunRecurringTaskEveryPeriod() {
        TaskSchedulerApi s = scheduler();
        s.scheduleRecurring(10, 10, note("r"));

        clock.advance(10);
        assertThat(s.runDue()).isEqualTo(1);
        clock.advance(5);
        assertThat(s.runDue()).isZero();
        clock.advance(5);
        assertThat(s.runDue()).isEqualTo(1);

        assertThat(log).containsExactly("r", "r");
    }

    @Test
    void shouldRefuseRecurringTaskWithNonPositivePeriod() {
        TaskSchedulerApi s = scheduler();

        assertThat(s.scheduleRecurring(0, 0, note("r"))).isEmpty();
        assertThat(s.scheduleRecurring(0, -5, note("r"))).isEmpty();

        assertThat(s.runDue()).isZero();
    }

    @Test
    void shouldCatchUpEveryMissedRecurringOccurrence() {
        TaskSchedulerApi s = scheduler();
        s.scheduleRecurring(100, 10, note("r"));   // due 1100, 1110, 1120, ...

        clock.advance(135);                         // now 1135: 1100, 1110, 1120, 1130 are missed
        assertThat(s.runDue()).isEqualTo(4);
        assertThat(s.runDue()).isZero();
        clock.advance(5);                           // now 1140
        assertThat(s.runDue()).isEqualTo(1);
    }

    @Test
    void shouldKeepFixedRateWhenTheHostIsLate() {
        TaskSchedulerApi s = scheduler();
        s.scheduleRecurring(0, 100, note("r"));     // due 1000, 1100, 1200, 1300, ...

        assertThat(s.runDue()).isEqualTo(1);        // 1000
        clock.advance(250);                         // now 1250: 1100, 1200
        assertThat(s.runDue()).isEqualTo(2);
        clock.advance(49);                          // now 1299
        assertThat(s.runDue()).isZero();
        clock.advance(1);                           // now 1300, not 1350: next due = previous due + period
        assertThat(s.runDue()).isEqualTo(1);
    }

    @Test
    void shouldInterleaveMissedOccurrencesWithOtherTasksByDueTime() {
        TaskSchedulerApi s = scheduler();
        s.scheduleRecurring(10, 10, note("r"));     // 1010, 1020, 1030
        s.scheduleOnce(25, note("o"));              // 1025

        clock.advance(30);

        assertThat(s.runDue()).isEqualTo(4);
        assertThat(log).containsExactly("r", "r", "o", "r");
    }

    @Test
    void shouldNotRunCancelledOneShotTask() {
        TaskSchedulerApi s = scheduler();
        String id = s.scheduleOnce(10, note("a"));

        assertThat(s.cancel(id)).isTrue();
        clock.advance(10);

        assertThat(s.runDue()).isZero();
        assertThat(log).isEmpty();
    }

    @Test
    void shouldStopRecurringTaskWhenCancelled() {
        TaskSchedulerApi s = scheduler();
        String id = s.scheduleRecurring(10, 10, note("r")).orElseThrow();
        clock.advance(10);
        s.runDue();

        assertThat(s.cancel(id)).isTrue();
        clock.advance(100);

        assertThat(s.runDue()).isZero();
        assertThat(log).containsExactly("r");
    }

    @Test
    void shouldRefuseToCancelUnknownFinishedOrAlreadyCancelledTasks() {
        TaskSchedulerApi s = scheduler();
        String done = s.scheduleOnce(0, note("done"));
        s.runDue();
        String cancelled = s.scheduleOnce(10, note("x"));
        s.cancel(cancelled);

        assertThat(s.cancel("nope")).isFalse();
        assertThat(s.cancel(done)).isFalse();
        assertThat(s.cancel(cancelled)).isFalse();
    }

    @Test
    void shouldPreventRunOfATaskThatIsDueButNotRunYet() {
        TaskSchedulerApi s = scheduler();
        String a = s.scheduleOnce(10, note("a"));
        s.scheduleOnce(10, note("b"));
        clock.advance(50);                          // both are overdue, runDue has not been called

        assertThat(s.cancel(a)).isTrue();

        assertThat(s.runDue()).isEqualTo(1);
        assertThat(log).containsExactly("b");
    }

    @Test
    void shouldContinueWithOtherTasksWhenATaskThrows() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(0, () -> {
            throw new IllegalStateException("boom");
        });
        s.scheduleOnce(0, note("after"));

        assertThat(s.runDue()).isEqualTo(2);
        assertThat(log).containsExactly("after");
    }

    @Test
    void shouldKeepARecurringTaskRecurringAfterItThrows() {
        TaskSchedulerApi s = scheduler();
        AtomicInteger attempts = new AtomicInteger();
        s.scheduleRecurring(0, 10, () -> {
            attempts.incrementAndGet();
            throw new IllegalStateException("boom");
        });

        assertThat(s.runDue()).isEqualTo(1);
        clock.advance(20);
        assertThat(s.runDue()).isEqualTo(2);

        assertThat(attempts.get()).isEqualTo(3);
    }

    @Test
    void shouldLetATaskScheduleFurtherWork() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(0, () -> {
            log.add("outer");
            s.scheduleOnce(100, note("later"));
        });

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            assertThat(s.runDue()).isEqualTo(1);
        });
        assertThat(s.runDue()).isZero();
        clock.advance(100);
        assertThat(s.runDue()).isEqualTo(1);

        assertThat(log).containsExactly("outer", "later");
    }

    @Test
    void shouldRunWorkScheduledFromInsideATaskInTheSameCallWhenAlreadyDue() {
        TaskSchedulerApi s = scheduler();
        s.scheduleOnce(0, () -> {
            log.add("outer");
            s.scheduleOnce(0, note("inner"));
        });
        s.scheduleOnce(0, note("sibling"));

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            assertThat(s.runDue()).isEqualTo(3);
        });
        assertThat(log).containsExactly("outer", "sibling", "inner");
    }

    @Test
    void shouldLetATaskCancelAnotherDueTask() {
        TaskSchedulerApi s = scheduler();
        AtomicReference<String> victim = new AtomicReference<>();
        AtomicBoolean cancelled = new AtomicBoolean();
        s.scheduleOnce(10, () -> cancelled.set(s.cancel(victim.get())));
        victim.set(s.scheduleOnce(20, note("victim")));
        clock.advance(30);

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            assertThat(s.runDue()).isEqualTo(1);
        });

        assertThat(cancelled).isTrue();
        assertThat(log).isEmpty();
    }

    @Test
    void shouldLetARecurringTaskCancelItselfAndStopTheCatchUp() {
        TaskSchedulerApi s = scheduler();
        AtomicReference<String> self = new AtomicReference<>();
        AtomicInteger runs = new AtomicInteger();
        AtomicBoolean cancelResult = new AtomicBoolean();
        self.set(s.scheduleRecurring(10, 10, () -> {
            if (runs.incrementAndGet() == 3) {
                cancelResult.set(s.cancel(self.get()));
            }
        }).orElseThrow());
        clock.advance(100);                         // ten occurrences are missed

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            assertThat(s.runDue()).isEqualTo(3);
        });

        assertThat(cancelResult).isTrue();
        clock.advance(100);
        assertThat(s.runDue()).isZero();
        assertThat(runs.get()).isEqualTo(3);
    }

    @Test
    void shouldRunEveryTaskExactlyOnceWhenManyThreadsSubmit() {
        TaskSchedulerApi s = scheduler();
        int threads = 16;
        int perThread = 100;
        AtomicIntegerArray runs = new AtomicIntegerArray(threads * perThread);

        List<List<String>> ids = Concurrently.collect(threads, t -> {
            List<String> mine = new ArrayList<>();
            for (int k = 0; k < perThread; k++) {
                int n = t * perThread + k;
                mine.add(s.scheduleOnce(k % 3, () -> runs.incrementAndGet(n)));
            }
            return mine;
        });
        clock.advance(10);

        assertThat(ids.stream().flatMap(List::stream).distinct().count()).isEqualTo(threads * perThread);
        assertThat(s.runDue()).isEqualTo(threads * perThread);
        for (int n = 0; n < runs.length(); n++) {
            assertThat(runs.get(n)).as("task %d", n).isEqualTo(1);
        }
    }

    @Test
    void shouldRunEachOccurrenceOnceWhenManyThreadsCallRunDue() {
        TaskSchedulerApi s = scheduler();
        int threads = 8;
        int perThread = 50;
        AtomicIntegerArray runs = new AtomicIntegerArray(threads * perThread);

        List<Integer> executed = Concurrently.collect(threads, t -> {
            int mine = 0;
            for (int k = 0; k < perThread; k++) {
                int n = t * perThread + k;
                s.scheduleOnce(0, () -> runs.incrementAndGet(n));
                mine += s.runDue();
            }
            return mine;
        });

        int total = executed.stream().mapToInt(Integer::intValue).sum() + s.runDue();
        assertThat(total).isEqualTo(threads * perThread);
        for (int n = 0; n < runs.length(); n++) {
            assertThat(runs.get(n)).as("task %d", n).isEqualTo(1);
        }
    }

    @Test
    void shouldNotRunATaskThatWasCancelledConcurrentlyAfterCancelReturnedTrue() {
        TaskSchedulerApi s = scheduler();
        int n = 400;
        AtomicIntegerArray runs = new AtomicIntegerArray(n);
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < n; i++) {
            int k = i;
            ids.add(s.scheduleOnce(0, () -> runs.incrementAndGet(k)));
        }

        List<Boolean> cancelled = Concurrently.collect(2, t -> {
            if (t == 0) {
                s.runDue();
                return null;
            }
            // Cancels in reverse so that it contends with the runner.
            boolean all = true;
            for (int i = n - 1; i >= n / 2; i--) {
                if (s.cancel(ids.get(i))) {
                    if (runs.get(i) != 0) {
                        all = false; // cancelled yet it had already run
                    }
                }
            }
            return all;
        });

        assertThat(cancelled.get(1)).isTrue();
        for (int i = 0; i < n; i++) {
            assertThat(runs.get(i)).isLessThanOrEqualTo(1);
        }
    }

    @Test
    void shouldHandleManyPendingTasksWithoutScanningThemAllOnEveryCall() {
        TaskSchedulerApi s = scheduler();
        int n = 50_000;
        AtomicInteger executed = new AtomicInteger();
        assertTimeoutPreemptively(Duration.ofSeconds(30), () -> {
            for (int i = 1; i <= n; i++) {
                s.scheduleOnce(i, executed::incrementAndGet);
            }
            for (int i = 1; i <= n; i++) {
                clock.advance(1);
                assertThat(s.runDue()).isEqualTo(1);
            }
        });
        assertThat(executed.get()).isEqualTo(n);
    }
}
