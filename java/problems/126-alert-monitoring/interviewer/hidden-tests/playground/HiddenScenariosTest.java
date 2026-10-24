package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.AlertMonitor;
import contract.Notifier;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    private static final long T0 = 5_000_000L;

    private final FakeClock clock = new FakeClock(T0);
    private final List<String> log = Collections.synchronizedList(new ArrayList<>());
    private final Notifier notifier = (ch, rule, state, sev, value, t) -> log.add(ch + "|" + rule + "|" + state + "|" + t);

    private AlertMonitor monitor() {
        return Candidate.of(() -> Solution.create(notifier, clock));
    }

    private static void rule(AlertMonitor m, String id, String metric, String cmp, double th, long window,
                             String agg, String sev, long forMs, long cooldown) {
        m.defineRule(id, metric, cmp, th, window, agg, sev, forMs, cooldown);
    }

    @Test
    void shouldCountSuppressedRefiresAcrossAFlappingRule() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "flap", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 10_000);
        // 10 cycles of: fire (1 s window), resolve, ok. One cycle every 3 s.
        for (int i = 0; i < 10; i++) {
            m.record("x", 1);
            m.evaluate();
            clock.advance(1_000);
            m.evaluate();
            clock.advance(1_000);
            m.evaluate();
            clock.advance(1_000);
        }
        // The rule enters FIRING at 0, 3, 6, ... 27 s. With a 10 s cooldown the notified ones are 0, 12 and 24 s.
        long firings = log.stream().filter(l -> l.contains("|FIRING|")).count();
        long resolved = log.stream().filter(l -> l.contains("|RESOLVED|")).count();
        assertThat(firings).isEqualTo(3);
        assertThat(resolved).isEqualTo(10);
        assertThat(m.suppressedCount("flap")).isEqualTo(7);
        assertThat(log.get(0)).isEqualTo("c|flap|FIRING|" + T0);
    }

    @Test
    void shouldNotSpendTheCooldownOnASilencedFiring() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 60_000);
        m.silence("r", T0 + 1_000);
        m.record("x", 1);
        m.evaluate();                       // FIRING while silenced: nothing delivered, cooldown not started
        clock.advance(1_000);
        m.evaluate();                       // RESOLVED (silence over) delivered
        clock.advance(1_000);
        m.evaluate();                       // OK
        m.record("x", 1);
        m.evaluate();                       // FIRING at T0+2000: no earlier delivered FIRING -> delivered

        assertThat(log).containsExactly("c|r|RESOLVED|" + (T0 + 1_000), "c|r|FIRING|" + (T0 + 2_000));
        assertThat(m.suppressedCount("r")).isEqualTo(0);
    }

    @Test
    void shouldNeverSendTheFiringNotificationLateWhenASilenceEndsWhileStillFiring() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "x", "GT", 0, 100_000, "COUNT", "CRITICAL", 0, 0);
        m.silence("r", T0 + 10_000);
        m.record("x", 1);
        m.evaluate();
        clock.advance(20_000);
        m.evaluate();                       // still firing, silence over: no catch-up notification
        assertThat(log).isEmpty();

        clock.advance(100_000);
        m.evaluate();
        assertThat(log).containsExactly("c|r|RESOLVED|" + (T0 + 120_000));
    }

    @Test
    void shouldShrinkRetentionWhenTheLargestWindowRuleIsDeleted() {
        AlertMonitor m = monitor();
        rule(m, "short", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 0);
        rule(m, "long", "x", "GT", 0, 60_000, "COUNT", "WARN", 0, 0);
        for (int i = 0; i < 10; i++) {
            m.record("x", 1);
            clock.advance(1_000);
        }
        m.evaluate();
        assertThat(m.sampleCount("x")).isEqualTo(10);

        m.deleteRule("long");
        m.evaluate();
        assertThat(m.sampleCount("x")).isEqualTo(0);   // all samples are >= 1000 ms old
    }

    @Test
    void shouldKeepSamplesOfMetricsThatNoRuleWatchesAndKeepRetentionPerMetric() {
        AlertMonitor m = monitor();
        rule(m, "r", "watched", "GT", 0, 1_000, "COUNT", "WARN", 0, 0);
        m.record("watched", 1);
        m.record("unwatched", 1);
        clock.advance(5_000);
        m.evaluate();

        assertThat(m.sampleCount("watched")).isEqualTo(0);
        assertThat(m.sampleCount("unwatched")).isEqualTo(1);
    }

    @Test
    void shouldEvaluateANewRuleAgainstSamplesRecordedBeforeItExisted() {
        AlertMonitor m = monitor();
        m.record("x", 10);
        clock.advance(500);
        m.record("x", 30);
        rule(m, "late", "x", "GTE", 20, 1_000, "AVG", "WARN", 0, 0);
        m.registerChannel("c", "INFO");
        m.evaluate();

        assertThat(m.state("late")).isEqualTo("FIRING");
        assertThat(log).containsExactly("c|late|FIRING|" + (T0 + 500));
    }

    @Test
    void shouldCountASampleRecordedAtTheSameMillisecondAsTheEvaluation() {
        AlertMonitor m = monitor();
        rule(m, "r", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 0);
        clock.advance(7);
        m.record("x", 1);
        m.evaluate();

        assertThat(m.state("r")).isEqualTo("FIRING");
    }

    @Test
    void shouldResetThePendingPeriodWhenTheConditionBrieflyDropsAndComesBack() {
        AlertMonitor m = monitor();
        rule(m, "r", "x", "GT", 10, 2_000, "MAX", "WARN", 5_000, 0);
        m.record("x", 50);
        m.evaluate();                       // PENDING since T0
        clock.advance(2_000);
        m.evaluate();                       // sample expired: OK
        clock.advance(1_000);
        m.record("x", 50);
        m.evaluate();                       // PENDING again since T0+3000
        clock.advance(2_000);
        m.record("x", 50);
        m.evaluate();                       // 2000 < 5000
        clock.advance(2_000);
        m.record("x", 50);
        m.evaluate();                       // 4000 < 5000
        assertThat(m.state("r")).isEqualTo("PENDING");
        clock.advance(1_000);
        m.record("x", 50);
        m.evaluate();                       // 5000 >= 5000

        assertThat(m.state("r")).isEqualTo("FIRING");
        assertThat(m.history("r")).containsExactly(
            T0 + ":PENDING", (T0 + 2_000) + ":OK", (T0 + 3_000) + ":PENDING", (T0 + 8_000) + ":FIRING");
    }

    @Test
    void shouldHandleNegativeValuesAndFractions() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "cold", "temp", "LT", -4.5, 10_000, "AVG", "INFO", 0, 0);
        m.record("temp", -3);
        m.record("temp", -6.5);   // avg -4.75
        m.evaluate();

        assertThat(m.state("cold")).isEqualTo("FIRING");
        m.record("temp", 10);     // avg (-3-6.5+10)/3 = 0.1666...
        clock.advance(1);
        m.evaluate();
        assertThat(m.state("cold")).isEqualTo("RESOLVED");
    }

    @Test
    void shouldNotNotifyForARuleDeletedWhileFiringAndStartFreshWhenRedefined() {
        AlertMonitor m = monitor();
        m.registerChannel("c", "INFO");
        rule(m, "r", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 600_000);
        m.record("x", 1);
        m.evaluate();
        log.clear();

        m.deleteRule("r");
        assertThat(log).isEmpty();
        rule(m, "r", "x", "GT", 0, 1_000, "COUNT", "WARN", 0, 600_000);
        m.evaluate();                        // fresh rule: no cooldown carried over

        assertThat(log).containsExactly("c|r|FIRING|" + T0);
        assertThat(m.suppressedCount("r")).isEqualTo(0);
    }

    @Test
    void shouldHandleManyRulesAndChannelsWithExactDeliveryCounts() {
        AlertMonitor m = monitor();
        m.registerChannel("info", "INFO");
        m.registerChannel("warn", "WARN");
        m.registerChannel("crit", "CRITICAL");
        String[] sev = {"INFO", "WARN", "CRITICAL"};
        for (int i = 0; i < 300; i++) {
            rule(m, String.format("rule-%03d", i), "m" + (i % 5), "GT", i % 7, 10_000, "MAX", sev[i % 3], 0, 0);
        }
        for (int k = 0; k < 5; k++) {
            m.record("m" + k, 3);
        }
        m.evaluate();

        int firing = 0;
        int expectedDeliveries = 0;
        for (int i = 0; i < 300; i++) {
            if (3 > i % 7) {
                firing++;
                expectedDeliveries += 1 + (i % 3);  // INFO -> 1 channel, WARN -> 2, CRITICAL -> 3
            }
        }
        assertThat(m.activeAlerts()).hasSize(firing);
        assertThat(log).hasSize(expectedDeliveries);
        List<String> active = m.activeAlerts();
        assertThat(sevOf(active.get(0))).isEqualTo(2);
        assertThat(sevOf(active.get(active.size() - 1))).isEqualTo(0);
    }

    private static int sevOf(String id) {
        return Integer.parseInt(id.substring(5)) % 3;
    }

    @Test
    void shouldRetainEverySampleRecordedConcurrentlyAndPruneAfterwards() {
        AlertMonitor m = monitor();
        rule(m, "r", "x", "GT", 0, 5_000, "COUNT", "WARN", 0, 0);
        AtomicInteger evals = new AtomicInteger();
        Concurrently.run(8, t -> {
            for (int i = 0; i < 1_000; i++) {
                m.record("x", i);
                if (t == 7 && i % 100 == 0) {
                    m.evaluate();
                    evals.incrementAndGet();
                }
            }
        });

        assertThat(m.sampleCount("x")).isEqualTo(8_000);
        clock.advance(5_000);
        m.evaluate();
        assertThat(m.sampleCount("x")).isEqualTo(0);
        assertThat(evals.get()).isEqualTo(10);
    }
}
