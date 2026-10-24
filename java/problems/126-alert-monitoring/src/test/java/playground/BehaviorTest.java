package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AlertMonitor;
import contract.Notifier;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long T0 = 1_000_000L;

    /** Records every delivery as "channel|rule|STATE|SEVERITY|value|time"; channels in {@code failing} throw. */
    static final class Recorder implements Notifier {
        final List<String> log = Collections.synchronizedList(new ArrayList<>());
        final Set<String> failing = Collections.synchronizedSet(new HashSet<>());

        @Override
        public void notify(String channel, String rule, String state, String severity, double value, long timeMs) {
            if (failing.contains(channel)) {
                throw new IllegalStateException("channel down: " + channel);
            }
            log.add(channel + "|" + rule + "|" + state + "|" + severity + "|" + fmt(value) + "|" + timeMs);
        }

        List<String> take() {
            synchronized (log) {
                List<String> copy = new ArrayList<>(log);
                log.clear();
                return copy;
            }
        }
    }

    static String fmt(double v) {
        return v == Math.rint(v) ? String.valueOf((long) v) : String.valueOf(v);
    }

    private final FakeClock clock = new FakeClock(T0);
    private final Recorder rec = new Recorder();

    private AlertMonitor monitor() {
        return Candidate.of(() -> Solution.create(rec, clock));
    }

    /** rule(id, metric, comparator, threshold, windowMs, aggregation, severity, forMs, cooldownMs) */
    private static void rule(AlertMonitor m, String id, String metric, String cmp, double th, long window,
                             String agg, String sev, long forMs, long cooldown) {
        m.defineRule(id, metric, cmp, th, window, agg, sev, forMs, cooldown);
    }

    /** An immediate (forMs 0), no-cooldown rule over a 60 s window. */
    private static void simple(AlertMonitor m, String id, String metric, String cmp, double th, String agg) {
        rule(m, id, metric, cmp, th, 60_000, agg, AlertMonitor.WARN, 0, 0);
    }

    @Test
    void shouldStartInOkWithAnEmptyHistory() {
        AlertMonitor m = monitor();
        simple(m, "r1", "cpu", "GT", 80, "AVG");

        assertThat(m.state("r1")).isEqualTo("OK");
        assertThat(m.history("r1")).isEmpty();
        assertThat(m.activeAlerts()).isEmpty();
        assertThat(m.suppressedCount("r1")).isEqualTo(0);
    }

    @Test
    void shouldTreatGtAsStrictAndGteAsInclusive() {
        AlertMonitor m = monitor();
        simple(m, "gt", "cpu", "GT", 80, "MAX");
        simple(m, "gte", "cpu", "GTE", 80, "MAX");
        m.record("cpu", 80);
        m.evaluate();

        assertThat(m.state("gt")).isEqualTo("OK");
        assertThat(m.state("gte")).isEqualTo("FIRING");
    }

    @Test
    void shouldTreatLtAsStrictAndLteAsInclusive() {
        AlertMonitor m = monitor();
        simple(m, "lt", "mem", "LT", 10, "MIN");
        simple(m, "lte", "mem", "LTE", 10, "MIN");
        m.record("mem", 10);
        m.evaluate();

        assertThat(m.state("lt")).isEqualTo("OK");
        assertThat(m.state("lte")).isEqualTo("FIRING");
    }

    @Test
    void shouldComputeEveryAggregationOverTheWindow() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        for (String agg : List.of("AVG", "MAX", "MIN", "SUM", "COUNT")) {
            simple(m, agg, "lat", "GTE", 0, agg);
        }
        m.record("lat", 10);
        m.record("lat", 20);
        m.record("lat", 60);
        m.evaluate();

        assertThat(rec.take()).containsExactly(
            "c|AVG|FIRING|WARN|30|" + T0,
            "c|COUNT|FIRING|WARN|3|" + T0,
            "c|MAX|FIRING|WARN|60|" + T0,
            "c|MIN|FIRING|WARN|10|" + T0,
            "c|SUM|FIRING|WARN|90|" + T0);
    }

    @Test
    void shouldUseAHalfOpenWindowThatExcludesASampleExactlyWindowMsOld() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "q", "GT", 0, 1_000, "COUNT", "WARN", 0, 0);
        m.record("q", 1);          // at T0
        clock.advance(999);
        m.evaluate();              // sample is 999 ms old: inside (now-1000, now]
        assertThat(m.state("r")).isEqualTo("FIRING");

        clock.advance(1);
        m.evaluate();              // sample is exactly 1000 ms old: outside
        assertThat(m.state("r")).isEqualTo("RESOLVED");
    }

    @Test
    void shouldNotFireWithoutSamplesExceptForCountWhichIsZero() {
        AlertMonitor m = monitor();
        simple(m, "gt", "x", "GT", -5, "AVG");
        simple(m, "lt", "x", "LT", 5, "MIN");
        simple(m, "sumLt", "x", "LT", 5, "SUM");
        simple(m, "cntLt", "x", "LT", 1, "COUNT");
        simple(m, "cntGt", "x", "GT", 0, "COUNT");
        m.evaluate();

        assertThat(m.state("gt")).isEqualTo("OK");
        assertThat(m.state("lt")).isEqualTo("OK");
        assertThat(m.state("sumLt")).isEqualTo("OK");
        assertThat(m.state("cntLt")).isEqualTo("FIRING");
        assertThat(m.state("cntGt")).isEqualTo("OK");
    }

    @Test
    void shouldFireImmediatelyWhenForMsIsZero() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        simple(m, "r", "cpu", "GT", 80, "AVG");
        m.record("cpu", 95);
        m.evaluate();

        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(m.history("r")).containsExactly(T0 + ":FIRING");
        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|95|" + T0);
        assertThat(m.activeAlerts()).containsExactly("r");
    }

    @Test
    void shouldStayPendingUntilTheConditionHeldForForMs() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "cpu", "GT", 80, 60_000, "AVG", "WARN", 120_000, 0);
        m.record("cpu", 90);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("PENDING");

        clock.advance(60_000);
        m.record("cpu", 95);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("PENDING");
        assertThat(m.activeAlerts()).isEmpty();
        assertThat(rec.take()).isEmpty();

        clock.advance(59_999);
        m.record("cpu", 85);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("PENDING");

        clock.advance(1);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(m.history("r")).containsExactly(T0 + ":PENDING", (T0 + 120_000) + ":FIRING");
        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|85|" + (T0 + 120_000));
    }

    @Test
    void shouldReturnToOkWithoutNotifyingWhenTheConditionDropsBeforeForMs() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "cpu", "GT", 80, 10_000, "MAX", "CRITICAL", 30_000, 0);
        m.record("cpu", 99);
        m.evaluate();
        clock.advance(10_000);
        m.evaluate();   // sample left the window

        assertThat(m.state("r")).isEqualTo("OK");
        assertThat(m.history("r")).containsExactly(T0 + ":PENDING", (T0 + 10_000) + ":OK");
        assertThat(rec.take()).isEmpty();
    }

    @Test
    void shouldRestartThePendingPeriodAfterADropBackToOk() {
        AlertMonitor m = monitor();
        rule(m, "r", "cpu", "GT", 80, 10_000, "MAX", "WARN", 20_000, 0);
        m.record("cpu", 99);
        m.evaluate();                       // PENDING since T0
        clock.advance(10_000);
        m.evaluate();                       // OK
        clock.advance(5_000);
        m.record("cpu", 99);
        m.evaluate();                       // PENDING since T0+15000
        clock.advance(10_000);
        m.record("cpu", 99);
        m.evaluate();                       // only 10 s of the 20 s

        assertThat(m.state("r")).isEqualTo("PENDING");
        clock.advance(10_000);
        m.record("cpu", 99);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("FIRING");
    }

    @Test
    void shouldNotifyFiringOnlyOnTheTransition() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        simple(m, "r", "cpu", "GT", 80, "MAX");
        m.record("cpu", 90);
        m.evaluate();
        clock.advance(1_000);
        m.record("cpu", 91);
        m.evaluate();
        clock.advance(1_000);
        m.evaluate();

        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|90|" + T0);
        assertThat(m.history("r")).containsExactly(T0 + ":FIRING");
    }

    @Test
    void shouldResolveThenReturnToOkOnTheNextEvaluateThatStaysFalse() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "cpu", "GT", 80, 1_000, "MAX", "WARN", 0, 0);
        m.record("cpu", 90);
        m.evaluate();
        rec.take();

        clock.advance(1_000);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("RESOLVED");
        assertThat(m.activeAlerts()).isEmpty();
        assertThat(rec.take()).containsExactly("c|r|RESOLVED|WARN|0|" + (T0 + 1_000));

        clock.advance(1_000);
        m.evaluate();
        assertThat(m.state("r")).isEqualTo("OK");
        assertThat(rec.take()).isEmpty();
        assertThat(m.history("r")).containsExactly(T0 + ":FIRING", (T0 + 1_000) + ":RESOLVED",
            (T0 + 2_000) + ":OK");
    }

    @Test
    void shouldLeaveResolvedDirectlyForPendingOrFiringWhenTheConditionIsTrueAgain() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "imm", "a", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        rule(m, "slow", "b", "GT", 0, 1_000, "MAX", "WARN", 5_000, 0);
        m.record("a", 1);
        m.record("b", 1);
        m.evaluate();
        clock.advance(1_000);
        m.evaluate();                      // imm: RESOLVED, slow: back to OK
        assertThat(m.state("imm")).isEqualTo("RESOLVED");
        m.record("a", 1);
        m.evaluate();                      // true again without passing through OK
        assertThat(m.state("imm")).isEqualTo("FIRING");
        assertThat(m.history("imm")).containsExactly(T0 + ":FIRING", (T0 + 1_000) + ":RESOLVED",
            (T0 + 1_000) + ":FIRING");

        // slow went PENDING -> OK and never fired
        assertThat(m.history("slow")).containsExactly(T0 + ":PENDING", (T0 + 1_000) + ":OK");
    }

    @Test
    void shouldSuppressRefiringInsideTheCooldownAndCountIt() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "e", "GT", 0, 1_000, "COUNT", "WARN", 0, 10_000);
        m.record("e", 1);
        m.evaluate();                      // FIRING notified at T0
        clock.advance(1_000);
        m.evaluate();                      // RESOLVED
        clock.advance(1_000);
        m.evaluate();                      // OK
        clock.advance(1_000);
        m.record("e", 1);
        m.evaluate();                      // FIRING again at T0+3000: inside the 10 s cooldown

        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(m.suppressedCount("r")).isEqualTo(1);
        assertThat(rec.take()).containsExactly(
            "c|r|FIRING|WARN|1|" + T0,
            "c|r|RESOLVED|WARN|0|" + (T0 + 1_000));
    }

    @Test
    void shouldNotifyAgainOnceTheCooldownHasPassedExactly() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "e", "GT", 0, 1_000, "COUNT", "WARN", 0, 5_000);
        m.record("e", 1);
        m.evaluate();                      // notified at T0
        clock.advance(1_000);
        m.evaluate();                      // RESOLVED
        clock.advance(3_999);
        m.record("e", 1);
        m.evaluate();                      // FIRING at T0+4999: 4999 < 5000 suppressed
        assertThat(m.suppressedCount("r")).isEqualTo(1);

        clock.advance(1_000);
        m.evaluate();                      // RESOLVED at T0+5999
        clock.advance(1);
        m.evaluate();                      // OK at T0+6000
        m.record("e", 1);
        m.evaluate();                      // FIRING at T0+6000: 6000 >= 5000 since T0 -> notified
        assertThat(m.suppressedCount("r")).isEqualTo(1);
        assertThat(rec.take()).contains("c|r|FIRING|WARN|1|" + (T0 + 6_000));
    }

    @Test
    void shouldStillNotifyResolvedInsideTheCooldown() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "e", "GT", 0, 1_000, "COUNT", "WARN", 0, 1_000_000);
        m.record("e", 1);
        m.evaluate();
        clock.advance(1_000);
        m.evaluate();
        clock.advance(1);
        m.record("e", 1);
        m.evaluate();                      // suppressed re-fire
        clock.advance(1_000);
        m.evaluate();                      // RESOLVED again, notified

        assertThat(rec.take()).containsExactly(
            "c|r|FIRING|WARN|1|" + T0,
            "c|r|RESOLVED|WARN|0|" + (T0 + 1_000),
            "c|r|RESOLVED|WARN|0|" + (T0 + 2_001));
        assertThat(m.suppressedCount("r")).isEqualTo(1);
    }

    @Test
    void shouldRouteBySeverityToEachChannel() {
        AlertMonitor m = monitor();
        m.registerChannel("all", "INFO");
        m.registerChannel("pager", "CRITICAL");
        m.registerChannel("chat", "WARN");
        rule(m, "i", "x", "GT", 0, 1_000, "MAX", "INFO", 0, 0);
        rule(m, "w", "x", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        rule(m, "c", "x", "GT", 0, 1_000, "MAX", "CRITICAL", 0, 0);
        m.record("x", 1);
        m.evaluate();

        assertThat(rec.take()).containsExactly(
            "all|c|FIRING|CRITICAL|1|" + T0, "pager|c|FIRING|CRITICAL|1|" + T0, "chat|c|FIRING|CRITICAL|1|" + T0,
            "all|i|FIRING|INFO|1|" + T0,
            "all|w|FIRING|WARN|1|" + T0, "chat|w|FIRING|WARN|1|" + T0);
    }

    @Test
    void shouldDeliverInRuleIdOrderAndChannelRegistrationOrder() {
        AlertMonitor m = monitor();
        m.registerChannel("zeta", "INFO");
        m.registerChannel("alpha", "INFO");
        simple(m, "b", "x", "GT", 0, "MAX");
        simple(m, "a", "x", "GT", 0, "MAX");
        m.record("x", 1);
        m.evaluate();

        assertThat(rec.take()).containsExactly(
            "zeta|a|FIRING|WARN|1|" + T0, "alpha|a|FIRING|WARN|1|" + T0,
            "zeta|b|FIRING|WARN|1|" + T0, "alpha|b|FIRING|WARN|1|" + T0);
    }

    @Test
    void shouldKeepDeliveringWhenOneNotifierThrows() {
        AlertMonitor m = monitor();
        m.registerChannel("a", "INFO");
        m.registerChannel("broken", "INFO");
        m.registerChannel("z", "INFO");
        rec.failing.add("broken");
        simple(m, "r1", "x", "GT", 0, "MAX");
        simple(m, "r2", "x", "GT", 0, "MAX");
        m.record("x", 1);
        m.evaluate();

        assertThat(rec.take()).containsExactly(
            "a|r1|FIRING|WARN|1|" + T0, "z|r1|FIRING|WARN|1|" + T0,
            "a|r2|FIRING|WARN|1|" + T0, "z|r2|FIRING|WARN|1|" + T0);
        assertThat(m.state("r1")).isEqualTo("FIRING");
        assertThat(m.state("r2")).isEqualTo("FIRING");
    }

    @Test
    void shouldStartTheCooldownEvenWhenNoChannelReceivedTheFiringNotification() {
        AlertMonitor m = monitor();
        rule(m, "r", "e", "GT", 0, 1_000, "COUNT", "WARN", 0, 10_000);
        m.record("e", 1);
        m.evaluate();                      // no channels registered at all
        clock.advance(1_000);
        m.evaluate();
        clock.advance(1);
        m.record("e", 1);
        m.evaluate();

        assertThat(m.suppressedCount("r")).isEqualTo(1);
    }

    @Test
    void shouldWithholdNotificationsWhileSilencedButKeepTrackingState() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        simple(m, "r", "x", "GT", 0, "MAX");
        m.silence("r", T0 + 5_000);
        m.record("x", 1);
        m.evaluate();

        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(m.activeAlerts()).containsExactly("r");
        assertThat(m.history("r")).containsExactly(T0 + ":FIRING");
        assertThat(rec.take()).isEmpty();
        assertThat(m.suppressedCount("r")).isEqualTo(0);
    }

    @Test
    void shouldEndTheSilenceExactlyAtUntilMs() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "x", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        m.silence("r", T0 + 1_000);
        m.record("x", 1);
        m.evaluate();                      // silenced FIRING
        clock.advance(1_000);
        m.evaluate();                      // time == untilMs: no longer silenced; RESOLVED is delivered
        assertThat(rec.take()).containsExactly("c|r|RESOLVED|WARN|0|" + (T0 + 1_000));

        m.silence("r", T0 + 100_000);
        clock.advance(1_000);
        m.evaluate();                      // OK
        m.silence("r", 0);                 // a later call replaces the earlier one
        m.record("x", 1);
        m.evaluate();
        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|1|" + (T0 + 2_000));
    }

    @Test
    void shouldListActiveAlertsBySeverityDescendingThenRuleId() {
        AlertMonitor m = monitor();
        rule(m, "b-warn", "x", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        rule(m, "a-warn", "x", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        rule(m, "z-crit", "x", "GT", 0, 1_000, "MAX", "CRITICAL", 0, 0);
        rule(m, "m-info", "x", "GT", 0, 1_000, "MAX", "INFO", 0, 0);
        rule(m, "c-crit", "x", "GT", 0, 1_000, "MAX", "CRITICAL", 0, 0);
        rule(m, "pending", "x", "GT", 0, 1_000, "MAX", "CRITICAL", 5_000, 0);
        rule(m, "quiet", "x", "GT", 100, 1_000, "MAX", "CRITICAL", 0, 0);
        m.record("x", 1);
        m.evaluate();

        assertThat(m.activeAlerts()).containsExactly("c-crit", "z-crit", "a-warn", "b-warn", "m-info");
    }

    @Test
    void shouldKeepSamplesInsideTheLargestWindowOfTheMetricsRules() {
        AlertMonitor m = monitor();
        rule(m, "short", "x", "GT", 0, 1_000, "MAX", "WARN", 0, 0);
        rule(m, "long", "x", "GT", 0, 5_000, "MAX", "WARN", 0, 0);
        m.record("x", 1);
        clock.advance(2_000);
        m.record("x", 2);
        clock.advance(2_999);
        m.evaluate();                      // now = T0+4999: both samples are younger than 5000
        assertThat(m.sampleCount("x")).isEqualTo(2);

        clock.advance(1);
        m.evaluate();                      // now = T0+5000: the first sample is exactly 5000 old -> dropped
        assertThat(m.sampleCount("x")).isEqualTo(1);

        clock.advance(10_000);
        m.evaluate();
        assertThat(m.sampleCount("x")).isEqualTo(0);
    }

    @Test
    void shouldCountSamplesPerMetricAndReportZeroForUnknownMetrics() {
        AlertMonitor m = monitor();
        m.record("a", 1);
        m.record("a", 2);
        m.record("b", 3);

        assertThat(m.sampleCount("a")).isEqualTo(2);
        assertThat(m.sampleCount("b")).isEqualTo(1);
        assertThat(m.sampleCount("never")).isEqualTo(0);
    }

    @Test
    void shouldEvaluateRulesOnlyAgainstTheirOwnMetric() {
        AlertMonitor m = monitor();
        simple(m, "cpuRule", "cpu", "GT", 0, "MAX");
        simple(m, "memRule", "mem", "GT", 0, "MAX");
        m.record("cpu", 5);
        m.evaluate();

        assertThat(m.state("cpuRule")).isEqualTo("FIRING");
        assertThat(m.state("memRule")).isEqualTo("OK");
    }

    @Test
    void shouldForgetEverythingAboutADeletedRule() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        simple(m, "r", "x", "GT", 0, "MAX");
        m.record("x", 1);
        m.evaluate();
        rec.take();

        m.deleteRule("r");
        assertThat(m.activeAlerts()).isEmpty();
        assertThatThrownBy(() -> m.state("r")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.history("r")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.deleteRule("r")).isInstanceOf(IllegalArgumentException.class);

        simple(m, "r", "x", "GT", 0, "MAX");   // the id can be reused, with a fresh state
        assertThat(m.state("r")).isEqualTo("OK");
        assertThat(m.history("r")).isEmpty();
        m.evaluate();
        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|1|" + T0);
    }

    @Test
    void shouldRejectInvalidRuleDefinitionsAndChangeNothing() {
        AlertMonitor m = monitor();
        simple(m, "r", "x", "GT", 1, "MAX");
        Class<IllegalArgumentException> iae = IllegalArgumentException.class;

        assertThatThrownBy(() -> simple(m, "r", "y", "LT", 9, "MIN")).isInstanceOf(iae);   // duplicate id
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, 0, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, -5, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, 10, "MAX", "WARN", -1, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, 10, "MAX", "WARN", 0, -1)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "EQ", 1, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "gt", 1, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, 10, "MEDIAN", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", 1, 10, "MAX", "FATAL", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "x", "GT", Double.NaN, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, " ", "x", "GT", 1, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, null, "x", "GT", 1, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);
        assertThatThrownBy(() -> rule(m, "a", "", "GT", 1, 10, "MAX", "WARN", 0, 0)).isInstanceOf(iae);

        assertThatThrownBy(() -> m.state("a")).isInstanceOf(iae);
        m.record("x", 5);
        m.evaluate();
        assertThat(m.activeAlerts()).containsExactly("r");   // original rule untouched (GT 1 on x)
    }

    @Test
    void shouldRejectInvalidRecordsChannelsAndUnknownRules() {
        AlertMonitor m = monitor();
        Class<IllegalArgumentException> iae = IllegalArgumentException.class;
        m.registerChannel("c", "WARN");

        assertThatThrownBy(() -> m.record(null, 1)).isInstanceOf(iae);
        assertThatThrownBy(() -> m.record("  ", 1)).isInstanceOf(iae);
        assertThatThrownBy(() -> m.record("x", Double.NaN)).isInstanceOf(iae);
        assertThatThrownBy(() -> m.record("x", Double.POSITIVE_INFINITY)).isInstanceOf(iae);
        assertThat(m.sampleCount("x")).isEqualTo(0);

        assertThatThrownBy(() -> m.registerChannel("c", "INFO")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.registerChannel("", "INFO")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.registerChannel("d", "LOUD")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.registerChannel("d", null)).isInstanceOf(iae);

        assertThatThrownBy(() -> m.state("nope")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.history("nope")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.silence("nope", 5)).isInstanceOf(iae);
        assertThatThrownBy(() -> m.suppressedCount("nope")).isInstanceOf(iae);
        assertThatThrownBy(() -> m.deleteRule("nope")).isInstanceOf(iae);
    }

    @Test
    void shouldStampSamplesWithTheClockAtRecordTime() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "x", "GT", 0, 1_000, "SUM", "WARN", 0, 0);
        m.record("x", 1);
        clock.advance(500);
        m.record("x", 2);
        clock.advance(600);       // first sample is now 1100 ms old, the second 600 ms
        m.evaluate();

        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|2|" + (T0 + 1_100));
    }

    @Test
    void shouldNotLoseSamplesWhenRecordedFromManyThreads() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "hits", "GTE", 1, 60_000, "COUNT", "WARN", 0, 0);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 500; i++) {
                m.record("hits", 1);
            }
        });

        assertThat(m.sampleCount("hits")).isEqualTo(4_000);
        m.evaluate();
        assertThat(rec.take()).containsExactly("c|r|FIRING|WARN|4000|" + T0);
    }

    @Test
    void shouldStayConsistentWhenRecordingWhileEvaluating() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "hits", "GTE", 1, 60_000, "COUNT", "WARN", 0, 0);

        Concurrently.run(6, t -> {
            for (int i = 0; i < 300; i++) {
                if (t == 0) {
                    m.evaluate();
                } else {
                    m.record("hits", 1);
                }
            }
        });

        m.evaluate();
        assertThat(m.sampleCount("hits")).isEqualTo(1_500);
        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(rec.take()).hasSize(1);
    }
}
