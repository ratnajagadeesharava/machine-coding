package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LoggingApi;
import java.util.ArrayList;
import java.util.List;
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
}
