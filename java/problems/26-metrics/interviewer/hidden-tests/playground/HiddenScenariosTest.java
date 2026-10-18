package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.MetricsApi;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private final FakeClock clock = new FakeClock(1_000_000);

    private MetricsApi metrics() {
        return Candidate.of(() -> Solution.create(clock));
    }

    @Test
    void anEventExactlyAtTheLeftEdgeShouldBeExcludedAndOneAtNowIncluded() {
        MetricsApi m = metrics();
        m.increment("c", 1, Map.of());   // t = 1_000_000
        clock.advance(60_000);
        m.increment("c", 10, Map.of());  // t = 1_060_000 = now

        assertThat(m.countInWindow("c", 60_000)).isEqualTo(10);
        assertThat(m.countInWindow("c", 60_001)).isEqualTo(11);
    }

    @Test
    void percentilesOnRankBoundariesShouldUseCeilingRanks() {
        MetricsApi m = metrics();
        for (long v = 1; v <= 7; v++) {
            m.observe("h", v * 10);   // 10..70, n = 7
        }

        assertThat(m.percentile("h", 14)).hasValue(10);   // 14*7/100 = 0.98 -> rank 1
        assertThat(m.percentile("h", 15)).hasValue(20);   // 1.05 -> rank 2
        assertThat(m.percentile("h", 29)).hasValue(30);   // 2.03 -> rank 3
        assertThat(m.percentile("h", 57)).hasValue(40);   // 3.99 -> rank 4
        assertThat(m.percentile("h", 58)).hasValue(50);   // 4.06 -> rank 5
        assertThat(m.percentile("h", 100)).hasValue(70);
    }

    @Test
    void aSingleValueShouldAnswerEveryPercentile() {
        MetricsApi m = metrics();
        m.observe("h", 42);

        for (int p = 1; p <= 100; p++) {
            assertThat(m.percentile("h", p)).hasValue(42);
        }
    }

    @Test
    void theEmptyTagSetShouldBeIndependentOfTaggedCountersButIncludedInTheTotal() {
        MetricsApi m = metrics();
        m.increment("c", 2, Map.of());
        m.increment("c", 3, Map.of("a", "1"));

        assertThat(m.counter("c", Map.of())).isEqualTo(2);
        assertThat(m.counter("c", Map.of("a", "1"))).isEqualTo(3);
        assertThat(m.counterTotal("c")).isEqualTo(5);
    }

    @Test
    void refusedIncrementsShouldLeaveNoTraceInTotalsOrWindows() {
        MetricsApi m = metrics();
        m.increment("c", 5, Map.of());
        m.increment("c", 0, Map.of("a", "1"));
        m.increment("c", -9, Map.of());

        assertThat(m.counterTotal("c")).isEqualTo(5);
        assertThat(m.counter("c", Map.of("a", "1"))).isZero();
        assertThat(m.countInWindow("c", 10_000)).isEqualTo(5);
    }
}
