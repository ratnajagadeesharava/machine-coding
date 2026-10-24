package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.HitCounter;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldStayBoundedWhenHitsKeepArrivingAndNobodyReads() {
        FakeClock clock = new FakeClock(0);
        HitCounter c = Candidate.of(() -> Solution.create(60_000, clock));

        for (int t = 0; t < 300_000; t++) {
            clock.set(t);
            c.hit("stream");
        }

        assertThat(c.memoryProbe()).isLessThanOrEqualTo(100_000);
        assertThat(c.count("stream")).isEqualTo(60_000);
        assertThat(c.countInLast("stream", 1_000)).isEqualTo(1_000);
    }

    @Test
    void shouldReleaseTheMemoryOfManyKeysThatWentQuiet() {
        FakeClock clock = new FakeClock(0);
        HitCounter c = Candidate.of(() -> Solution.create(1_000, clock));
        for (int k = 0; k < 20_000; k++) {
            c.hit("key-" + k, 2);
            if (k % 100 == 99) {
                clock.advance(1);
            }
        }
        assertThat(c.totalInWindow()).isEqualTo(40_000);

        clock.advance(10_000);

        assertThat(c.totalInWindow()).isZero();
        assertThat(c.topKeys(5)).isEmpty();
        assertThat(c.memoryProbe()).isLessThanOrEqualTo(100_000);
    }

    @Test
    void shouldLayerShrinkAndGrowAcrossSeveralWindows() {
        FakeClock clock = new FakeClock(0);
        HitCounter c = Candidate.of(() -> Solution.create(10_000, clock));
        for (int t : new int[] {0, 3_000, 6_000, 9_000}) {
            clock.set(t);
            c.hit("a", t == 0 ? 1 : t);
        }
        assertThat(c.count("a")).isEqualTo(1 + 3_000 + 6_000 + 9_000);

        c.setWindow(5_000);
        assertThat(c.count("a")).isEqualTo(15_000);

        c.setWindow(20_000);
        assertThat(c.count("a")).as("hit at 3000 and 0 stay dropped").isEqualTo(15_000);
        clock.set(14_000);
        assertThat(c.count("a")).isEqualTo(15_000);
        clock.set(15_001);
        assertThat(c.count("a")).as("both survivors are still inside the 20000 window").isEqualTo(15_000);
        clock.set(26_000);
        assertThat(c.count("a")).as("6000 expired, 9000 live").isEqualTo(9_000);
        clock.set(29_000);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldHandleTheClockGoingBackAndForthAroundExpiry() {
        FakeClock clock = new FakeClock(10_000);
        HitCounter c = Candidate.of(() -> Solution.create(1_000, clock));
        c.hit("a");
        clock.set(0);
        c.hit("a", 3);
        c.hit("a", 3);
        clock.set(10_500);
        c.hit("a");

        assertThat(c.count("a")).isEqualTo(8);
        clock.set(10_700);
        assertThat(c.count("a")).isEqualTo(8);
        clock.set(100);
        assertThat(c.count("a")).as("now never went below 10700").isEqualTo(8);
        clock.set(11_000);
        assertThat(c.count("a")).isEqualTo(1);
        clock.set(11_500);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldKeepCountsExactWhileTheWindowIsResizedConcurrently() {
        FakeClock clock = new FakeClock(1_000_000);
        HitCounter c = Candidate.of(() -> Solution.create(5_000, clock));

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 20_000; i++) {
                    c.setWindow(1_000 + (i % 2) * 4_000);
                }
                return;
            }
            for (int i = 0; i < 20_000; i++) {
                c.hit("k" + (i % 5));
            }
        });

        assertThat(c.totalInWindow()).isEqualTo(160_000);
    }

    @Test
    void shouldAnswerTopKeysConsistentlyWhileWritersAreRunning() {
        FakeClock clock = new FakeClock(0);
        HitCounter c = Candidate.of(() -> Solution.create(60_000, clock));

        Concurrently.run(16, 60_000, t -> {
            if (t >= 12) {
                for (int i = 0; i < 2_000; i++) {
                    List<String> top = c.topKeys(3);
                    assertThat(top.size()).isLessThanOrEqualTo(3);
                    assertThat(c.countInLast("k0", 1)).isGreaterThanOrEqualTo(0);
                }
                return;
            }
            for (int i = 0; i < 10_000; i++) {
                c.hit("k" + t);
            }
        });

        assertThat(c.totalInWindow()).isEqualTo(120_000);
        assertThat(c.topKeys(3)).containsExactly("k0", "k1", "k10");
    }

    @Test
    void shouldKeepWorkingAcrossAVeryLongRunWithBigBatches() {
        FakeClock clock = new FakeClock(0);
        HitCounter c = Candidate.of(() -> Solution.create(3_600_000, clock));

        long expected = 0;
        for (int minute = 0; minute < 600; minute++) {
            clock.set(minute * 60_000L);
            c.hit("api", 1_000_000_000L);
            if (minute >= 540) {
                expected += 1_000_000_000L;
            }
        }
        clock.set(599 * 60_000L);

        assertThat(c.count("api")).isEqualTo(60_000_000_000L);
        assertThat(c.countInLast("api", 600_000)).isEqualTo(10_000_000_000L);
        assertThat(expected).isEqualTo(60_000_000_000L);
        assertThat(c.memoryProbe()).isLessThanOrEqualTo(100_000);
    }
}
