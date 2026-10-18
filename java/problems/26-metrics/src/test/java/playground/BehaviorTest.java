package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

import contract.MetricsApi;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.OptionalLong;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {
    private static final long T0 = 10_000;
    private static final Map<String, String> NONE = Map.of();

    private final FakeClock clock = new FakeClock(T0);

    private MetricsApi metrics() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private static void observeAll(MetricsApi m, String name, long... values) {
        for (long v : values) {
            m.observe(name, v);
        }
    }

    @Test
    void shouldCountIncrementsPerTagSet() {
        MetricsApi m = metrics();

        assertThat(m.increment("http", 1, Map.of("route", "/pay", "status", "200"))).isTrue();
        assertThat(m.increment("http", 2, Map.of("route", "/pay", "status", "500"))).isTrue();
        assertThat(m.increment("http", 4, Map.of("route", "/pay", "status", "200"))).isTrue();

        assertThat(m.counter("http", Map.of("route", "/pay", "status", "200"))).isEqualTo(5);
        assertThat(m.counter("http", Map.of("route", "/pay", "status", "500"))).isEqualTo(2);
        assertThat(m.counterTotal("http")).isEqualTo(7);
    }

    @Test
    void shouldMatchTagsExactly() {
        MetricsApi m = metrics();
        m.increment("http", 1, Map.of("route", "/pay", "status", "200"));
        m.increment("http", 10, Map.of("route", "/pay"));
        m.increment("http", 100, NONE);

        assertThat(m.counter("http", Map.of("route", "/pay"))).isEqualTo(10);
        assertThat(m.counter("http", Map.of("status", "200"))).isZero();
        assertThat(m.counter("http", Map.of("route", "/pay", "status", "404"))).isZero();
        assertThat(m.counter("http", NONE)).isEqualTo(100);
        assertThat(m.counterTotal("http")).isEqualTo(111);
    }

    @Test
    void shouldIgnoreTagOrderAndLaterChangesToTheCallersMap() {
        MetricsApi m = metrics();
        java.util.Map<String, String> tags = new java.util.LinkedHashMap<>();
        tags.put("a", "1");
        tags.put("b", "2");
        m.increment("x", 1, tags);
        tags.put("c", "3");
        tags.remove("a");

        assertThat(m.counter("x", Map.of("b", "2", "a", "1"))).isEqualTo(1);
        assertThat(m.counter("x", tags)).isZero();
    }

    @Test
    void shouldReturnZeroForUnknownCounters() {
        MetricsApi m = metrics();

        assertThat(m.counter("nope", NONE)).isZero();
        assertThat(m.counterTotal("nope")).isZero();
        assertThat(m.countInWindow("nope", 1_000)).isZero();
    }

    @Test
    void shouldRefuseNonPositiveDeltasAndBlankNames() {
        MetricsApi m = metrics();

        assertThat(m.increment("x", 0, NONE)).isFalse();
        assertThat(m.increment("x", -5, NONE)).isFalse();
        assertThat(m.increment("  ", 1, NONE)).isFalse();
        assertThat(m.observe("", 1)).isFalse();

        assertThat(m.counterTotal("x")).isZero();
        assertThat(m.countInWindow("x", 1_000_000)).isZero();
    }

    @Test
    void shouldComputeNearestRankPercentiles() {
        MetricsApi m = metrics();
        observeAll(m, "latency", 50, 15, 40, 20, 35);   // sorted: 15 20 35 40 50

        assertThat(m.percentile("latency", 1)).hasValue(15);
        assertThat(m.percentile("latency", 20)).hasValue(15);   // rank ceil(1.0) = 1
        assertThat(m.percentile("latency", 30)).hasValue(20);   // rank ceil(1.5) = 2
        assertThat(m.percentile("latency", 40)).hasValue(20);   // rank 2
        assertThat(m.percentile("latency", 50)).hasValue(35);   // rank ceil(2.5) = 3
        assertThat(m.percentile("latency", 75)).hasValue(40);   // rank ceil(3.75) = 4
        assertThat(m.percentile("latency", 95)).hasValue(50);   // rank ceil(4.75) = 5
        assertThat(m.percentile("latency", 100)).hasValue(50);
        assertThat(m.observationCount("latency")).isEqualTo(5);
    }

    @Test
    void shouldReturnTheValueWithTheSamePercentileOverAHundredDistinctValues() {
        MetricsApi m = metrics();
        List<Long> values = new ArrayList<>();
        for (long v = 1; v <= 100; v++) {
            values.add(v);
        }
        Collections.shuffle(values, new Random(7));
        for (long v : values) {
            m.observe("h", v);
        }

        for (int p = 1; p <= 100; p++) {
            assertThat(m.percentile("h", p)).as("p%d", p).hasValue(p);
        }
    }

    @Test
    void shouldHandleDuplicatesAndNegativeValues() {
        MetricsApi m = metrics();
        observeAll(m, "h", 5, 5, 5, -3, 5, 5, 5, 5, 5, 100);   // n=10: -3, 5 x8, 100

        assertThat(m.percentile("h", 10)).hasValue(-3);
        assertThat(m.percentile("h", 11)).hasValue(5);
        assertThat(m.percentile("h", 90)).hasValue(5);
        assertThat(m.percentile("h", 91)).hasValue(100);
    }

    @Test
    void shouldKeepPercentilesCorrectWhenValuesArriveBetweenQueries() {
        MetricsApi m = metrics();
        observeAll(m, "h", 10, 20, 30);
        assertThat(m.percentile("h", 100)).hasValue(30);

        m.observe("h", 5);
        m.observe("h", 40);

        assertThat(m.percentile("h", 100)).hasValue(40);
        assertThat(m.percentile("h", 20)).hasValue(5);     // n=5, rank 1
        assertThat(m.percentile("h", 60)).hasValue(20);    // rank 3
    }

    @Test
    void shouldReturnEmptyPercentileForUnknownHistogramsOrBadRanks() {
        MetricsApi m = metrics();
        m.observe("h", 1);

        assertThat(m.percentile("nope", 50)).isEqualTo(OptionalLong.empty());
        assertThat(m.percentile("h", 0)).isEqualTo(OptionalLong.empty());
        assertThat(m.percentile("h", 101)).isEqualTo(OptionalLong.empty());
        assertThat(m.percentile("h", -1)).isEqualTo(OptionalLong.empty());
        assertThat(m.observationCount("nope")).isZero();
    }

    @Test
    void shouldKeepCountersAndHistogramsInSeparateNamespaces() {
        MetricsApi m = metrics();
        m.increment("x", 3, NONE);
        m.observe("x", 99);

        assertThat(m.counterTotal("x")).isEqualTo(3);
        assertThat(m.observationCount("x")).isEqualTo(1);
        assertThat(m.percentile("x", 100)).hasValue(99);
    }

    @Test
    void shouldCountOnlyWhatFallsInTheHalfOpenWindow() {
        MetricsApi m = metrics();
        m.increment("c", 5, NONE);                    // at 10_000
        clock.advance(1_000);
        m.increment("c", 3, Map.of("k", "v"));        // at 11_000

        assertThat(m.countInWindow("c", 1_000)).isEqualTo(3);   // (10_000, 11_000]: the first is excluded
        assertThat(m.countInWindow("c", 1_001)).isEqualTo(8);   // (9_999, 11_000]
        assertThat(m.countInWindow("c", 1)).isEqualTo(3);
        assertThat(m.countInWindow("c", 0)).isZero();
        assertThat(m.countInWindow("c", -10)).isZero();
        assertThat(m.countInWindow("c", Long.MAX_VALUE)).isEqualTo(8);
    }

    @Test
    void shouldLetEventsLeaveTheWindowAsTimePasses() {
        MetricsApi m = metrics();
        m.increment("c", 1, NONE);
        clock.advance(400);
        m.increment("c", 2, NONE);
        clock.advance(400);
        m.increment("c", 4, NONE);

        assertThat(m.countInWindow("c", 1_000)).isEqualTo(7);
        clock.advance(300);                                     // now 11_100
        assertThat(m.countInWindow("c", 1_000)).isEqualTo(6);   // (10_100, 11_100]: drops the first
        clock.advance(700);                                     // now 11_800
        assertThat(m.countInWindow("c", 1_000)).isZero();       // (10_800, 11_800]: the last one is at 10_800, excluded
        assertThat(m.countInWindow("c", 1_001)).isEqualTo(4);   // (10_799, 11_800]
    }

    @Test
    void shouldSumAllTagSetsAndIgnoreOtherCountersInTheWindow() {
        MetricsApi m = metrics();
        m.increment("a", 1, Map.of("t", "1"));
        m.increment("a", 2, Map.of("t", "2"));
        m.increment("b", 100, NONE);

        assertThat(m.countInWindow("a", 5_000)).isEqualTo(3);
        assertThat(m.countInWindow("b", 5_000)).isEqualTo(100);
    }

    @Test
    void shouldNotLoseIncrementsFromManyThreads() {
        MetricsApi m = metrics();

        Concurrently.run(100, i -> {
            for (int k = 0; k < 1_000; k++) {
                m.increment("hits", 1, Map.of("shard", String.valueOf(i % 4)));
            }
        });

        assertThat(m.counterTotal("hits")).isEqualTo(100_000);
        for (int shard = 0; shard < 4; shard++) {
            assertThat(m.counter("hits", Map.of("shard", String.valueOf(shard)))).isEqualTo(25_000);
        }
        assertThat(m.countInWindow("hits", 1)).isEqualTo(100_000);
    }

    @Test
    void shouldNotLoseIncrementsOnASingleCounterFromManyThreads() {
        MetricsApi m = metrics();

        Concurrently.run(100, i -> {
            for (int k = 0; k < 1_000; k++) {
                m.increment("single", 1, NONE);
            }
        });

        assertThat(m.counter("single", NONE)).isEqualTo(100_000);
    }

    @Test
    void shouldObserveAndQueryPercentilesConcurrently() {
        MetricsApi m = metrics();

        Concurrently.run(12, t -> {
            if (t < 8) {
                for (long v = 1; v <= 1_000; v++) {
                    m.observe("lat", v);
                }
            } else {
                for (int k = 0; k < 300; k++) {
                    OptionalLong p = m.percentile("lat", 50);
                    if (p.isPresent()) {
                        assertThat(p.getAsLong()).isBetween(1L, 1_000L);
                    }
                    m.observationCount("lat");
                }
            }
        });

        assertThat(m.observationCount("lat")).isEqualTo(8_000);
        assertThat(m.percentile("lat", 50)).hasValue(500);
        assertThat(m.percentile("lat", 1)).hasValue(10);
        assertThat(m.percentile("lat", 100)).hasValue(1_000);
    }

    @Test
    void shouldStayFastWithAMillionObservationsAndInterleavedQueries() {
        MetricsApi m = metrics();
        assertTimeoutPreemptively(Duration.ofSeconds(60), () -> {
            long seed = 12345;
            for (int i = 0; i < 1_000_000; i++) {
                seed = (seed * 6364136223846793005L + 1442695040888963407L);
                m.observe("big", (seed >>> 33) % 100_000);
            }
            assertThat(m.observationCount("big")).isEqualTo(1_000_000);
            for (int q = 0; q < 1_000; q++) {
                assertThat(m.percentile("big", 1 + q % 100)).isPresent();
            }
            for (int round = 0; round < 300; round++) {
                m.observe("big", 100_000 + round);
                assertThat(m.percentile("big", 99)).isPresent();
            }
            assertThat(m.percentile("big", 100)).hasValue(100_299);
            assertThat(m.observationCount("big")).isEqualTo(1_000_300);
        });
    }

    @Test
    void shouldAnswerWindowQueriesQuicklyOverManyIncrements() {
        MetricsApi m = metrics();
        assertTimeoutPreemptively(Duration.ofSeconds(60), () -> {
            for (int i = 0; i < 300_000; i++) {
                m.increment("events", 1, Map.of("n", String.valueOf(i % 10)));
                clock.advance(1);
            }
            long total = 0;
            for (int q = 1; q <= 5_000; q++) {
                total += m.countInWindow("events", q * 50L);
            }
            assertThat(m.countInWindow("events", 1_000)).isEqualTo(999);   // events at t = now-999 .. now-1
            assertThat(m.countInWindow("events", 10_000_000)).isEqualTo(300_000);
            assertThat(total).isPositive();
        });
    }
}
