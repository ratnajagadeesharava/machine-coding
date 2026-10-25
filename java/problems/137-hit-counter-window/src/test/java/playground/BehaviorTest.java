package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.HitCounter;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private final FakeClock clock = new FakeClock(10_000);

    private HitCounter counter(long windowMs) {
        return Candidate.of(() -> Solution.create(windowMs, clock));
    }

    @Test
    void shouldCountHitsInsideTheWindow() {
        HitCounter c = counter(1_000);

        c.hit("a");
        clock.advance(100);
        c.hit("a");
        c.hit("b");

        assertThat(c.count("a")).isEqualTo(2);
        assertThat(c.count("b")).isEqualTo(1);
    }

    @Test
    void shouldCountAnUnknownKeyAsZero() {
        HitCounter c = counter(1_000);
        c.hit("a");

        assertThat(c.count("nobody")).isZero();
        assertThat(c.countInLast("nobody", 500)).isZero();
        assertThat(counter(1_000).totalInWindow()).isZero();
    }

    @Test
    void shouldExpireAHitExactlyWindowMsOld() {
        HitCounter c = counter(1_000);
        c.hit("a");

        clock.advance(999);
        assertThat(c.count("a")).isEqualTo(1);
        clock.advance(1);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldRecordSeveralHitsWithOneCall() {
        HitCounter c = counter(1_000);

        c.hit("a", 5);
        c.hit("a");
        c.hit("a", 1_000_000_000_000L);

        assertThat(c.count("a")).isEqualTo(1_000_000_000_006L);
        assertThat(c.totalInWindow()).isEqualTo(1_000_000_000_006L);
    }

    @Test
    void shouldExpireTheOldHitsAndKeepTheNewOnes() {
        HitCounter c = counter(1_000);
        c.hit("a", 3);
        clock.advance(600);
        c.hit("a", 2);

        clock.advance(500);

        assertThat(c.count("a")).isEqualTo(2);
        clock.advance(500);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldRejectInvalidArguments() {
        HitCounter c = counter(1_000);

        assertThatThrownBy(() -> Solution.create(0, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-5, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(1_000, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.hit(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.hit(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.hit("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.hit("a", -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.hit("", 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.count(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.count("  ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.countInLast(" ", 10)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.setWindow(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.setWindow(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topKeys(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topKeys(-2)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldChangeNothingWhenAnArgumentIsRejected() {
        HitCounter c = counter(1_000);
        c.hit("a");

        assertThatThrownBy(() -> c.hit("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.setWindow(0)).isInstanceOf(IllegalArgumentException.class);

        assertThat(c.count("a")).isEqualTo(1);
        assertThat(c.window()).isEqualTo(1_000);
    }

    @Test
    void shouldAnswerShorterQueriesWithCountInLast() {
        HitCounter c = counter(60_000);
        c.hit("a", 4);
        clock.advance(30_000);
        c.hit("a", 2);
        clock.advance(25_000);
        c.hit("a", 1);

        assertThat(c.countInLast("a", 5_000)).isEqualTo(1);
        assertThat(c.countInLast("a", 25_000)).isEqualTo(1);
        assertThat(c.countInLast("a", 25_001)).isEqualTo(3);
        assertThat(c.countInLast("a", 60_000)).isEqualTo(7);
        assertThat(c.count("a")).isEqualTo(7);
    }

    @Test
    void shouldTreatAHitExactlyDurationOldAsOutsideCountInLast() {
        HitCounter c = counter(10_000);
        c.hit("a");
        clock.advance(5_000);

        assertThat(c.countInLast("a", 5_000)).isZero();
        assertThat(c.countInLast("a", 5_001)).isEqualTo(1);
    }

    @Test
    void shouldRejectDurationsOutsideZeroToWindow() {
        HitCounter c = counter(1_000);
        c.hit("a");

        assertThatThrownBy(() -> c.countInLast("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.countInLast("a", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.countInLast("a", 1_001)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.countInLast("unknown", 1_001)).isInstanceOf(IllegalArgumentException.class);
        assertThat(c.countInLast("a", 1_000)).isEqualTo(1);
        assertThat(c.countInLast("a", 1)).isEqualTo(1);
    }

    @Test
    void shouldSumTheWindowedHitsOfAllKeys() {
        HitCounter c = counter(1_000);
        c.hit("a", 2);
        c.hit("b", 3);
        clock.advance(600);
        c.hit("c", 4);

        assertThat(c.totalInWindow()).isEqualTo(9);
        clock.advance(500);
        assertThat(c.totalInWindow()).isEqualTo(4);
        clock.advance(500);
        assertThat(c.totalInWindow()).isZero();
    }

    @Test
    void shouldWorkWithAOneMillisecondWindow() {
        HitCounter c = counter(1);

        c.hit("a", 2);
        assertThat(c.count("a")).isEqualTo(2);
        assertThat(c.countInLast("a", 1)).isEqualTo(2);
        clock.advance(1);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldDropHitsOutsideTheNewWindowWhenShrinkingAndNotBringThemBack() {
        HitCounter c = counter(10_000);
        c.hit("a", 1);
        clock.advance(3_000);
        c.hit("a", 10);
        clock.advance(3_000);
        c.hit("a", 100);
        assertThat(c.count("a")).isEqualTo(111);

        c.setWindow(4_000);
        assertThat(c.window()).isEqualTo(4_000);
        assertThat(c.count("a")).isEqualTo(110);

        c.setWindow(10_000);
        assertThat(c.window()).isEqualTo(10_000);
        assertThat(c.count("a")).as("the dropped hit stays dropped").isEqualTo(110);
        assertThat(c.totalInWindow()).isEqualTo(110);
    }

    @Test
    void shouldKeepTheLiveHitsAndAcceptLongerQueriesWhenGrowing() {
        HitCounter c = counter(1_000);
        c.hit("a", 2);
        clock.advance(500);

        c.setWindow(5_000);

        assertThat(c.count("a")).isEqualTo(2);
        clock.advance(3_000);
        assertThat(c.countInLast("a", 4_000)).isEqualTo(2);
        assertThat(c.count("a")).isEqualTo(2);
        clock.advance(1_500);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldNotResurrectHitsThatExpiredBeforeTheWindowGrew() {
        HitCounter c = counter(1_000);
        c.hit("a", 7);
        clock.advance(1_500);

        c.setWindow(5_000);

        assertThat(c.count("a")).isZero();
        assertThat(c.totalInWindow()).isZero();
    }

    @Test
    void shouldApplyTheNewWindowLimitToCountInLast() {
        HitCounter c = counter(1_000);
        c.hit("a");
        c.setWindow(500);

        assertThatThrownBy(() -> c.countInLast("a", 501)).isInstanceOf(IllegalArgumentException.class);
        assertThat(c.countInLast("a", 500)).isEqualTo(1);
        c.setWindow(2_000);
        assertThat(c.countInLast("a", 2_000)).isEqualTo(1);
    }

    @Test
    void shouldRankTopKeysByCountThenByKey() {
        HitCounter c = counter(1_000);
        c.hit("pear", 3);
        c.hit("apple", 3);
        c.hit("zebra", 9);
        c.hit("mango", 1);
        c.hit("fig", 3);

        assertThat(c.topKeys(10)).containsExactly("zebra", "apple", "fig", "pear", "mango");
        assertThat(c.topKeys(2)).containsExactly("zebra", "apple");
        assertThat(c.topKeys(1)).containsExactly("zebra");
    }

    @Test
    void shouldListOnlyKeysWithHitsInTheWindowInTopKeys() {
        HitCounter c = counter(1_000);
        assertThat(c.topKeys(5)).isEmpty();

        c.hit("old", 100);
        clock.advance(600);
        c.hit("new", 1);
        assertThat(c.topKeys(5)).containsExactly("old", "new");

        clock.advance(500);
        assertThat(c.topKeys(5)).containsExactly("new");
        clock.advance(500);
        assertThat(c.topKeys(5)).isEmpty();
    }

    @Test
    void shouldReturnAListTheCallerMayModify() {
        HitCounter c = counter(1_000);
        c.hit("a");

        List<String> top = c.topKeys(3);
        top.add("junk");

        assertThat(c.topKeys(3)).containsExactly("a");
    }

    @Test
    void shouldRecordHitsAtTheLastSeenTimeWhenTheClockMovesBackwards() {
        HitCounter c = counter(1_000);
        c.hit("a");

        clock.set(9_000);
        c.hit("a");
        c.hit("a", 2);

        assertThat(c.count("a")).as("all hits were recorded at 10000").isEqualTo(4);
        clock.set(10_999);
        assertThat(c.count("a")).isEqualTo(4);
        clock.set(11_000);
        assertThat(c.count("a")).isZero();
    }

    @Test
    void shouldAnswerQueriesAtTheLastSeenTimeWhenTheClockMovesBackwards() {
        HitCounter c = counter(1_000);
        clock.set(20_000);
        c.hit("a");

        clock.set(5_000);

        assertThat(c.count("a")).isEqualTo(1);
        assertThat(c.totalInWindow()).isEqualTo(1);
        assertThat(c.topKeys(1)).containsExactly("a");
        clock.set(20_500);
        assertThat(c.count("a")).isEqualTo(1);
        clock.set(5_000);
        c.hit("a");
        assertThat(c.count("a")).as("now stayed at 20500").isEqualTo(2);
        clock.set(21_000);
        assertThat(c.count("a")).as("only the hit at 20500 is left").isEqualTo(1);
    }

    @Test
    void shouldHaveNothingStoredBeforeTheFirstHit() {
        assertThat(counter(1_000).memoryProbe()).isZero();
    }

    @Test
    void shouldNotKeepOneRecordPerHit() {
        HitCounter c = counter(2_000);
        for (int ms = 0; ms < 1_000; ms++) {
            for (int i = 0; i < 5_000; i++) {
                c.hit("busy");
            }
            clock.advance(1);
        }

        assertThat(c.count("busy")).isEqualTo(5_000_000);
        assertThat(c.memoryProbe()).isLessThanOrEqualTo(100_000);

        clock.advance(5_000);

        assertThat(c.count("busy")).isZero();
        assertThat(c.memoryProbe()).isLessThanOrEqualTo(100_000);
    }

    @Test
    void shouldLoseNoHitWhenManyThreadsHitTheSameKey() {
        HitCounter c = counter(60_000);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 10_000; i++) {
                c.hit("shared");
            }
        });

        assertThat(c.count("shared")).isEqualTo(160_000);
        assertThat(c.totalInWindow()).isEqualTo(160_000);
    }

    @Test
    void shouldLoseNoHitWhenManyThreadsHitManyKeysWithBatches() {
        HitCounter c = counter(60_000);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 2_000; i++) {
                c.hit("k" + (i % 10), 1 + (t % 3));
            }
        });

        long expected = 0;
        for (int t = 0; t < 16; t++) {
            expected += 2_000L * (1 + (t % 3));
        }
        assertThat(c.totalInWindow()).isEqualTo(expected);
        long sum = 0;
        for (int k = 0; k < 10; k++) {
            sum += c.count("k" + k);
        }
        assertThat(sum).isEqualTo(expected);
    }

    @Test
    void shouldStayCorrectOver50Runs() {
        for (int run = 0; run < 50; run++) {
            FakeClock own = new FakeClock(0);
            HitCounter c = Candidate.of(() -> Solution.create(1_000, own));

            Concurrently.run(16, t -> {
                for (int i = 0; i < 200; i++) {
                    c.hit("k" + (t % 4));
                }
            });

            assertThat(c.totalInWindow()).as("run %d", run).isEqualTo(3_200);
            assertThat(c.count("k0")).as("run %d", run).isEqualTo(800);
        }
    }
}
