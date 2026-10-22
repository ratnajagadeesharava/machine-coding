package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LoggingApi;
import java.util.ArrayList;
import java.util.List;
import java.util.Collections;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class BehaviorTest {
    private static final long T0 = 1_700_000_000_123L;

    private final FakeClock clock = new FakeClock(T0);
    private final List<String> lines = new ArrayList<>();

    private LoggingApi logging() {
        return Candidate.of(() -> Solution.create(clock));
    }

    /** A logging system with one appender "main" (min level TRACE) collecting into {@code lines}. */
    private LoggingApi collecting() {
        LoggingApi api = logging();
        assertThat(api.addAppender("main", lines::add, "TRACE")).isTrue();
        return api;
    }

    @Test
    void shouldFormatLineWithTimestampLevelLoggerAndMessage() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "started");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - started");
    }

    @Test
    void shouldUseTheClockAtTheMomentOfEachCall() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "one");
        clock.advance(7);
        api.log("app", "ERROR", "two");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - one", "1700000000130 [ERROR] app - two");
    }

    @Test
    void shouldSubstitutePlaceholdersInOrder() {
        LoggingApi api = collecting();

        api.log("app.db", "WARN", "slow query {}ms on {}", "850", "orders");

        assertThat(lines).containsExactly("1700000000123 [WARN] app.db - slow query 850ms on orders");
    }

    @Test
    void shouldLeaveUnfilledPlaceholdersWhenArgumentsAreMissing() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "a={} b={} c={}", "1");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - a=1 b={} c={}");
    }

    @Test
    void shouldIgnoreExtraArguments() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "only {}", "x", "y", "z");
        api.log("app", "INFO", "none", "x");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - only x", "1700000000123 [INFO] app - none");
    }

    @Test
    void shouldNotRescanSubstitutedArgumentsForPlaceholders() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "x={} y={}", "{}", "2");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - x={} y=2");
    }

    @Test
    void shouldCopyAnythingThatIsNotExactlyAPlaceholder() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "{ } { } } {{}} $1 \\ {}", "A");

        assertThat(lines).containsExactly("1700000000123 [INFO] app - { } { } } {A} $1 \\ {}");
    }

    @Test
    void shouldDefaultTheRootThresholdToInfo() {
        LoggingApi api = collecting();

        api.log("app", "TRACE", "t");
        api.log("app", "DEBUG", "d");
        api.log("app", "INFO", "i");
        api.log("app", "WARN", "w");
        api.log("app", "ERROR", "e");

        assertThat(lines).hasSize(3);
        assertThat(lines.get(0)).endsWith("[INFO] app - i");
        assertThat(lines.get(1)).endsWith("[WARN] app - w");
        assertThat(lines.get(2)).endsWith("[ERROR] app - e");
    }

    @Test
    void shouldRankLevelsFromTraceToError() {
        LoggingApi api = collecting();
        api.setLevel("root", "TRACE");
        String[] levels = {"TRACE", "DEBUG", "INFO", "WARN", "ERROR"};

        for (int threshold = 0; threshold < levels.length; threshold++) {
            lines.clear();
            api.setLevel("app", levels[threshold]);
            for (String level : levels) {
                api.log("app", level, "m");
            }
            assertThat(lines).as("threshold %s", levels[threshold]).hasSize(levels.length - threshold);
        }
    }

    @Test
    void shouldApplyAThresholdSetOnTheRootToLoggersWithoutOwnLevel() {
        LoggingApi api = collecting();

        assertThat(api.setLevel("root", "ERROR")).isTrue();
        api.log("app", "WARN", "dropped");
        api.log("app", "ERROR", "kept");

        assertThat(lines).hasSize(1);
        assertThat(lines.get(0)).endsWith("[ERROR] app - kept");
    }

    @Test
    void shouldInheritTheThresholdOfTheNearestConfiguredAncestor() {
        LoggingApi api = collecting();
        api.setLevel("app", "ERROR");

        api.log("app.db", "WARN", "dropped");
        api.log("app.db.pool", "WARN", "dropped too");
        api.log("app.db.pool", "ERROR", "kept");

        assertThat(lines).hasSize(1);
        assertThat(lines.get(0)).endsWith("[ERROR] app.db.pool - kept");
    }

    @Test
    void shouldLetAMoreSpecificLoggerOverrideItsAncestors() {
        LoggingApi api = collecting();
        api.setLevel("app", "ERROR");
        api.setLevel("app.db", "DEBUG");

        api.log("app", "DEBUG", "dropped");
        api.log("app.db", "DEBUG", "kept");
        api.log("app.db.pool", "DEBUG", "kept too");

        assertThat(lines).hasSize(2);
        assertThat(lines.get(0)).endsWith("[DEBUG] app.db - kept");
        assertThat(lines.get(1)).endsWith("[DEBUG] app.db.pool - kept too");
    }

    @Test
    void shouldTreatOnlyWholeSegmentsAsParents() {
        LoggingApi api = collecting();
        api.setLevel("app", "ERROR");

        api.log("application", "INFO", "kept: not a child of app");
        api.log("app.db", "INFO", "dropped: child of app");

        assertThat(lines).hasSize(1);
        assertThat(lines.get(0)).contains("application");
    }

    @Test
    void shouldSeeAThresholdChangeImmediatelyEvenAfterTheLoggerWasUsed() {
        LoggingApi api = collecting();
        api.log("app.db", "DEBUG", "dropped");
        assertThat(lines).isEmpty();

        api.setLevel("app", "DEBUG");
        api.log("app.db", "DEBUG", "kept");
        api.setLevel("app", "WARN");
        api.log("app.db", "DEBUG", "dropped again");

        assertThat(lines).hasSize(1);
        assertThat(lines.get(0)).endsWith("kept");
    }

    @Test
    void shouldFilterEachAppenderByItsOwnMinimumLevelOnTopOfTheLoggerThreshold() {
        LoggingApi api = logging();
        List<String> warnings = new ArrayList<>();
        List<String> everything = new ArrayList<>();
        api.addAppender("warnings", warnings::add, "WARN");
        api.addAppender("everything", everything::add, "TRACE");
        api.setLevel("root", "DEBUG");

        api.log("app", "TRACE", "below logger threshold");
        api.log("app", "DEBUG", "d");
        api.log("app", "WARN", "w");

        assertThat(warnings).hasSize(1);
        assertThat(warnings.get(0)).endsWith("[WARN] app - w");
        assertThat(everything).hasSize(2);
    }

    @Test
    void shouldDeliverToEveryAppenderInRegistrationOrder() {
        LoggingApi api = logging();
        List<String> order = new ArrayList<>();
        api.addAppender("b", line -> order.add("b"), "TRACE");
        api.addAppender("a", line -> order.add("a"), "TRACE");
        api.addAppender("c", line -> order.add("c"), "TRACE");

        api.log("app", "INFO", "x");

        assertThat(order).containsExactly("b", "a", "c");
    }

    @Test
    void shouldRefuseInvalidOrDuplicateAppenders() {
        LoggingApi api = logging();
        List<String> first = new ArrayList<>();
        List<String> second = new ArrayList<>();

        assertThat(api.addAppender("x", first::add, "INFO")).isTrue();
        assertThat(api.addAppender("x", second::add, "INFO")).isFalse();
        assertThat(api.addAppender(" ", second::add, "INFO")).isFalse();
        assertThat(api.addAppender("y", second::add, "LOUD")).isFalse();

        api.log("app", "INFO", "m");
        assertThat(first).hasSize(1);
        assertThat(second).isEmpty();
    }

    @Test
    void shouldStopDeliveringAfterRemovalAndAllowReusingTheName() {
        LoggingApi api = logging();
        List<String> first = new ArrayList<>();
        List<String> second = new ArrayList<>();
        api.addAppender("x", first::add, "INFO");

        assertThat(api.removeAppender("x")).isTrue();
        assertThat(api.removeAppender("x")).isFalse();
        assertThat(api.removeAppender("unknown")).isFalse();
        api.log("app", "INFO", "lost");
        assertThat(api.addAppender("x", second::add, "INFO")).isTrue();
        api.log("app", "INFO", "seen");

        assertThat(first).isEmpty();
        assertThat(second).hasSize(1);
        assertThat(second.get(0)).endsWith("seen");
    }

    @Test
    void shouldNotShowEarlierMessagesToALateAppender() {
        LoggingApi api = logging();
        api.log("app", "INFO", "before");
        List<String> late = new ArrayList<>();

        api.addAppender("late", late::add, "TRACE");
        api.log("app", "INFO", "after");

        assertThat(late).hasSize(1);
        assertThat(late.get(0)).endsWith("after");
    }

    @Test
    void shouldKeepDeliveringToOtherAppendersWhenASinkFails() {
        LoggingApi api = logging();
        List<String> before = new ArrayList<>();
        List<String> after = new ArrayList<>();
        api.addAppender("before", before::add, "TRACE");
        api.addAppender("broken", line -> {
            throw new IllegalStateException("disk full");
        }, "TRACE");
        api.addAppender("after", after::add, "TRACE");

        api.log("app", "INFO", "one");
        api.log("app", "INFO", "two");

        assertThat(before).hasSize(2);
        assertThat(after).hasSize(2);
    }

    @Test
    void shouldIgnoreInvalidLevelsAndBlankLoggerNames() {
        LoggingApi api = collecting();

        assertThat(api.setLevel("app", "LOUD")).isFalse();
        assertThat(api.setLevel("app", "info")).isFalse();
        assertThat(api.setLevel(" ", "INFO")).isFalse();
        api.log("app", "LOUD", "x");
        api.log("app", "info", "x");
        api.log("", "INFO", "x");
        api.log("  ", "INFO", "x");

        assertThat(lines).isEmpty();
        api.log("app", "INFO", "valid");
        assertThat(lines).hasSize(1);
    }

    @Test
    void shouldGoNowhereWithoutAppenders() {
        LoggingApi api = logging();

        api.log("app", "ERROR", "nobody listens");

        assertThat(lines).isEmpty();
    }

    @Test
    void shouldBeFastWhenMostMessagesAreBelowTheThreshold() {
        LoggingApi api = collecting();
        api.setLevel("a.b.c.d.e.f", "WARN");
        long start = System.nanoTime();

        for (int i = 0; i < 300_000; i++) {
            api.log("a.b.c.d.e.f.g", "DEBUG", "item {} of {}", "x", "y");
        }
        for (int i = 0; i < 100_000; i++) {
            api.log("a.b.c.d.e.f.g", "ERROR", "item {} of {}", "x", "y");
        }

        assertThat(lines).hasSize(100_000);
        assertThat(System.nanoTime() - start).isLessThan(20_000_000_000L);
    }

    @Test
    void shouldFormatAnEmptyMessageAndEmptyArguments() {
        LoggingApi api = collecting();

        api.log("app", "INFO", "");
        api.log("app", "INFO", "[{}]", "");
        api.log("app", "INFO", "{}{}{}", "a", "b", "c");

        assertThat(lines).containsExactly(
                "1700000000123 [INFO] app - ",
                "1700000000123 [INFO] app - []",
                "1700000000123 [INFO] app - abc");
    }

    @Test
    void shouldLogUnderTheRootLoggerItselfUsingTheRootThreshold() {
        LoggingApi api = collecting();

        api.log("root", "DEBUG", "dropped");
        api.log("root", "INFO", "kept");
        api.setLevel("root", "DEBUG");
        api.log("root", "DEBUG", "kept too");

        assertThat(lines).containsExactly(
                "1700000000123 [INFO] root - kept", "1700000000123 [DEBUG] root - kept too");
    }

    @Test
    void shouldKeepAnOwnLevelWhenTheRootThresholdChanges() {
        LoggingApi api = collecting();
        api.setLevel("app", "DEBUG");

        api.setLevel("root", "ERROR");
        api.log("app.db", "DEBUG", "own level of app still applies");
        api.log("other", "WARN", "root now ERROR: dropped");

        assertThat(lines).hasSize(1);
        assertThat(lines.get(0)).endsWith("[DEBUG] app.db - own level of app still applies");
    }

    @Test
    void shouldNotLetAChildThresholdAffectItsParentOrSiblings() {
        LoggingApi api = collecting();
        api.setLevel("app.db", "ERROR");

        api.log("app", "INFO", "parent kept");
        api.log("app.web", "INFO", "sibling kept");
        api.log("app.db", "INFO", "child dropped");

        assertThat(lines).hasSize(2);
        assertThat(lines.get(0)).endsWith("app - parent kept");
        assertThat(lines.get(1)).endsWith("app.web - sibling kept");
    }

    @Test
    void shouldDeliverToAnAppenderWhoseMinimumEqualsTheMessageLevelButNotToHigherOnes() {
        LoggingApi api = logging();
        List<String> errorsOnly = new ArrayList<>();
        api.addAppender("errors", errorsOnly::add, "ERROR");

        api.log("app", "WARN", "w");
        api.log("app", "ERROR", "e");

        assertThat(errorsOnly).hasSize(1);
        assertThat(errorsOnly.get(0)).endsWith("[ERROR] app - e");
    }

    @Test
    void shouldPutAReAddedAppenderAtTheEndOfTheDeliveryOrder() {
        LoggingApi api = logging();
        List<String> order = new ArrayList<>();
        api.addAppender("a", line -> order.add("a"), "TRACE");
        api.addAppender("b", line -> order.add("b"), "TRACE");
        api.addAppender("c", line -> order.add("c"), "TRACE");

        assertThat(api.removeAppender("a")).isTrue();
        api.log("app", "INFO", "x");
        assertThat(order).containsExactly("b", "c");

        order.clear();
        assertThat(api.addAppender("a", line -> order.add("a"), "TRACE")).isTrue();
        api.log("app", "INFO", "y");
        assertThat(order).containsExactly("b", "c", "a");
    }

    @Test
    void shouldKeepDeliveringWhenTheFirstOrLastSinkFailsWithAnyExceptionType() {
        LoggingApi api = logging();
        List<String> middle = new ArrayList<>();
        api.addAppender("first", line -> {
            throw new UnsupportedOperationException("first");
        }, "TRACE");
        api.addAppender("middle", middle::add, "TRACE");
        api.addAppender("last", line -> {
            throw new IllegalArgumentException("last");
        }, "TRACE");

        api.log("app", "INFO", "one");
        api.log("app", "ERROR", "two");

        assertThat(middle).hasSize(2);
    }

    @Test
    void shouldStillDeliverToASinkThatFailedOnAnEarlierMessage() {
        LoggingApi api = logging();
        List<String> seen = new ArrayList<>();
        AtomicInteger calls = new AtomicInteger();
        api.addAppender("flaky", line -> {
            if (calls.getAndIncrement() == 0) {
                throw new IllegalStateException("first call fails");
            }
            seen.add(line);
        }, "TRACE");

        api.log("app", "INFO", "one");
        api.log("app", "INFO", "two");

        assertThat(calls.get()).isEqualTo(2);
        assertThat(seen).hasSize(1);
        assertThat(seen.get(0)).endsWith("two");
    }

    private static final int THREADS = 8;
    private static final int PER_THREAD = 5_000;

    /** Runs the tasks at the same time on separate threads and rethrows the first failure. */
    private void runConcurrently(List<Runnable> tasks) throws Exception {
        ExecutorService pool = Executors.newFixedThreadPool(tasks.size());
        try {
            CountDownLatch go = new CountDownLatch(1);
            List<Future<?>> futures = new ArrayList<>();
            for (Runnable task : tasks) {
                futures.add(pool.submit(() -> {
                    go.await();
                    task.run();
                    return null;
                }));
            }
            go.countDown();
            for (Future<?> f : futures) {
                f.get(60, TimeUnit.SECONDS);
            }
        } finally {
            pool.shutdownNow();
        }
    }

    private List<Runnable> producers(LoggingApi api, int threads, int perThread, String level) {
        List<Runnable> tasks = new ArrayList<>();
        for (int t = 0; t < threads; t++) {
            int id = t;
            tasks.add(() -> {
                for (int i = 0; i < perThread; i++) {
                    api.log("app.worker" + id, level, "t={} i={}", String.valueOf(id), String.valueOf(i));
                }
            });
        }
        return tasks;
    }

    private void assertEveryThreadMessageArrivedOnceInOrder(List<String> received, int threads, int perThread) {
        int[] next = new int[threads];
        Set<String> unique = new HashSet<>();
        for (String line : received) {
            assertThat(line).matches("1700000000123 \\[WARN\\] app\\.worker\\d+ - t=\\d+ i=\\d+");
            assertThat(unique.add(line)).as("duplicate line %s", line).isTrue();
            String[] parts = line.substring(line.indexOf("t=")).split("[ =]");
            int t = Integer.parseInt(parts[1]);
            int i = Integer.parseInt(parts[3]);
            assertThat(i).as("order within thread %d", t).isEqualTo(next[t]);
            next[t]++;
        }
        for (int t = 0; t < threads; t++) {
            assertThat(next[t]).as("messages of thread %d", t).isEqualTo(perThread);
        }
    }

    @Test
    void shouldLoseNothingAndKeepPerThreadOrderWhenManyThreadsLog() throws Exception {
        LoggingApi api = logging();
        List<String> received = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("main", received::add, "TRACE");

        runConcurrently(producers(api, THREADS, PER_THREAD, "WARN"));

        assertThat(received).hasSize(THREADS * PER_THREAD);
        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(received), THREADS, PER_THREAD);
    }

    @Test
    void shouldDeliverTheFullStreamToEveryAppenderWhenManyThreadsLog() throws Exception {
        LoggingApi api = logging();
        List<String> first = Collections.synchronizedList(new ArrayList<>());
        List<String> second = Collections.synchronizedList(new ArrayList<>());
        List<String> warningsOnly = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("first", first::add, "TRACE");
        api.addAppender("second", second::add, "INFO");
        api.addAppender("errors", warningsOnly::add, "ERROR");

        runConcurrently(producers(api, THREADS, 2_000, "WARN"));

        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(first), THREADS, 2_000);
        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(second), THREADS, 2_000);
        assertThat(warningsOnly).isEmpty();
    }

    @Test
    void shouldApplyLevelFilteringConsistentlyWhenManyThreadsLog() throws Exception {
        LoggingApi api = logging();
        api.setLevel("app", "WARN");
        List<Runnable> tasks = new ArrayList<>();
        for (int t = 0; t < THREADS; t++) {
            tasks.add(() -> {
                for (int i = 0; i < 2_000; i++) {
                    api.log("app.db", "DEBUG", "dropped");
                    api.log("app.db", "ERROR", "kept");
                }
            });
        }
        List<String> synced = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("sync", synced::add, "TRACE");

        runConcurrently(tasks);

        assertThat(synced).hasSize(THREADS * 2_000).allMatch(l -> l.endsWith("[ERROR] app.db - kept"));
    }

    @Test
    void shouldKeepDeliveringToHealthySinksWhenOneSinkFailsUnderConcurrentLogging() throws Exception {
        LoggingApi api = logging();
        List<String> received = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("broken", line -> {
            throw new IllegalStateException("disk full");
        }, "TRACE");
        api.addAppender("healthy", received::add, "TRACE");

        runConcurrently(producers(api, THREADS, 2_000, "WARN"));

        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(received), THREADS, 2_000);
    }

    @Test
    void shouldNotLoseMessagesWhileTheThresholdIsChangedConcurrently() throws Exception {
        LoggingApi api = logging();
        List<String> received = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("main", received::add, "TRACE");
        AtomicBoolean stop = new AtomicBoolean();
        List<Runnable> tasks = producers(api, 4, 5_000, "WARN");
        Thread flipper = new Thread(() -> {
            String[] levels = {"TRACE", "DEBUG", "INFO", "WARN"};
            int n = 0;
            while (!stop.get()) {
                api.setLevel("app", levels[n++ % levels.length]);
                api.setLevel("root", levels[n % levels.length]);
            }
        });
        flipper.start();
        try {
            runConcurrently(tasks);
        } finally {
            stop.set(true);
            flipper.join();
        }

        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(received), 4, 5_000);
    }

    @Test
    void shouldAddAndRemoveAppendersWhileOtherThreadsLogWithoutFailingOrDisturbingStableAppenders() throws Exception {
        LoggingApi api = logging();
        List<String> stable = Collections.synchronizedList(new ArrayList<>());
        api.addAppender("stable", stable::add, "TRACE");
        List<String> churned = Collections.synchronizedList(new ArrayList<>());
        AtomicBoolean stop = new AtomicBoolean();
        AtomicInteger added = new AtomicInteger();
        AtomicInteger removed = new AtomicInteger();
        Thread churn = new Thread(() -> {
            while (!stop.get()) {
                if (api.addAppender("churn", churned::add, "TRACE")) {
                    added.incrementAndGet();
                }
                if (api.removeAppender("churn")) {
                    removed.incrementAndGet();
                }
            }
        });
        churn.start();
        try {
            runConcurrently(producers(api, 6, 5_000, "WARN"));
        } finally {
            stop.set(true);
            churn.join();
        }

        assertEveryThreadMessageArrivedOnceInOrder(new ArrayList<>(stable), 6, 5_000);
        assertThat(added.get()).isEqualTo(removed.get());
        assertThat(api.addAppender("churn", churned::add, "TRACE")).isTrue();
        List<String> seen = new ArrayList<>(churned);
        assertThat(seen).hasSizeLessThanOrEqualTo(6 * 5_000);
        assertThat(new HashSet<>(seen)).hasSameSizeAs(seen);
        api.log("app", "WARN", "after churn");
        assertThat(churned.get(churned.size() - 1)).endsWith("after churn");
    }

    @Test
    void shouldAllowOnlyOneThreadToRegisterTheSameAppenderName() throws Exception {
        LoggingApi api = logging();
        AtomicInteger winners = new AtomicInteger();
        List<Runnable> tasks = new ArrayList<>();
        for (int t = 0; t < 16; t++) {
            tasks.add(() -> {
                if (api.addAppender("only-one", line -> { }, "INFO")) {
                    winners.incrementAndGet();
                }
            });
        }

        runConcurrently(tasks);

        assertThat(winners.get()).isEqualTo(1);
        assertThat(api.removeAppender("only-one")).isTrue();
        assertThat(api.removeAppender("only-one")).isFalse();
    }
}
