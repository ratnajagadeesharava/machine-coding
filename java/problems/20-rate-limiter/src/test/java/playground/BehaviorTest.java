package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.RateLimiterApi;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.atomic.AtomicIntegerArray;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {
    private static final long START = 1_000_000L;

    private final FakeClock clock = new FakeClock(START);

    private RateLimiterApi limiter(int limit, long windowMillis) {
        return Candidate.of(() -> Solution.create(limit, windowMillis, clock));
    }

    @Test
    void shouldAllowUpToTheLimitAndThenDeny() {
        RateLimiterApi rl = limiter(3, 1000);

        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isFalse();
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldKeepKeysIndependent() {
        RateLimiterApi rl = limiter(1, 1000);

        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isFalse();

        assertThat(rl.allow("b")).isTrue();
        assertThat(rl.allow("b")).isFalse();
    }

    @Test
    void shouldTreatKeysAsCaseSensitiveAndAcceptAnyString() {
        RateLimiterApi rl = limiter(1, 1000);

        assertThat(rl.allow("user")).isTrue();
        assertThat(rl.allow("USER")).isTrue();
        assertThat(rl.allow("")).isTrue();
        assertThat(rl.allow("é中")).isTrue();
        assertThat(rl.allow("")).isFalse();
        assertThat(rl.allow("user")).isFalse();
    }

    @Test
    void shouldAllowAgainWhenTheOldestRequestLeavesTheWindow() {
        RateLimiterApi rl = limiter(2, 1000);

        assertThat(rl.allow("a")).isTrue();      // t = 0
        clock.advance(400);
        assertThat(rl.allow("a")).isTrue();      // t = 400
        clock.advance(599);
        assertThat(rl.allow("a")).isFalse();     // t = 999: window (-1, 999] holds both
        clock.advance(1);
        assertThat(rl.allow("a")).isTrue();      // t = 1000: window (0, 1000] holds only t = 400
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldExpireARequestExactlyOneWindowAfterItWasAllowed() {
        RateLimiterApi rl = limiter(1, 1000);

        assertThat(rl.allow("a")).isTrue();
        clock.advance(999);
        assertThat(rl.allow("a")).isFalse();
        clock.advance(1);
        assertThat(rl.allow("a")).isTrue();
    }

    @Test
    void shouldNotCountDeniedRequests() {
        RateLimiterApi rl = limiter(2, 1000);

        assertThat(rl.allow("a")).isTrue();      // t = 0
        clock.advance(400);
        assertThat(rl.allow("a")).isTrue();      // t = 400
        clock.advance(100);
        assertThat(rl.allow("a")).isFalse();     // t = 500, denied
        clock.advance(499);
        assertThat(rl.allow("a")).isFalse();     // t = 999, denied
        clock.advance(1);
        assertThat(rl.allow("a")).isTrue();      // t = 1000: only t = 400 counts
        assertThat(rl.allow("a")).isFalse();
        clock.advance(400);
        assertThat(rl.allow("a")).isTrue();      // t = 1400: t = 400 left, t = 1000 counts
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldSlideInsteadOfResettingOnFixedBoundaries() {
        RateLimiterApi rl = limiter(2, 1000);

        clock.advance(900);
        assertThat(rl.allow("a")).isTrue();      // t = 900
        clock.advance(50);
        assertThat(rl.allow("a")).isTrue();      // t = 950
        clock.advance(50);
        assertThat(rl.allow("a")).isFalse();     // t = 1000: a fixed window would have reset here
        clock.advance(100);
        assertThat(rl.allow("a")).isFalse();     // t = 1100
        clock.advance(800);
        assertThat(rl.allow("a")).isTrue();      // t = 1900: only t = 950 counts
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldRestoreTheFullBudgetAfterAFullWindowOfSilence() {
        RateLimiterApi rl = limiter(5, 1000);
        for (int i = 0; i < 5; i++) {
            assertThat(rl.allow("a")).isTrue();
        }
        assertThat(rl.allow("a")).isFalse();

        clock.advance(1000);

        for (int i = 0; i < 5; i++) {
            assertThat(rl.allow("a")).isTrue();
        }
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldRejectInvalidConfiguration() {
        limiter(1, 1000); // skipped while not implemented
        assertThatThrownBy(() -> Solution.create(0, 1000, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-1, 1000, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(1, 0, clock)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAllowExactlyTheLimitWhenManyThreadsHammerOneKeyAtTheSameInstant() {
        RateLimiterApi rl = limiter(100, 60_000);

        List<Integer> allowedPerThread = Concurrently.collect(32, t -> {
            int allowed = 0;
            for (int i = 0; i < 50; i++) {
                if (rl.allow("hot")) {
                    allowed++;
                }
            }
            return allowed;
        });

        assertThat(allowedPerThread.stream().mapToInt(Integer::intValue).sum()).isEqualTo(100);
        assertThat(rl.allow("hot")).isFalse();
    }

    @Test
    void shouldHoldTheLimitPerKeyWhenManyKeysAreHammeredConcurrently() {
        RateLimiterApi rl = limiter(3, 60_000);
        AtomicIntegerArray allowed = new AtomicIntegerArray(100);

        Concurrently.run(8, t -> {
            for (int round = 0; round < 5; round++) {
                for (int k = 0; k < 100; k++) {
                    if (rl.allow("key" + k)) {
                        allowed.incrementAndGet(k);
                    }
                }
            }
        });

        for (int k = 0; k < 100; k++) {
            assertThat(allowed.get(k)).as("key%d", k).isEqualTo(3);
        }
    }

    @Test
    void shouldAllowExactlyTheLimitAgainInEachNewWindowUnderConcurrency() {
        RateLimiterApi rl = limiter(20, 1000);

        for (int phase = 0; phase < 3; phase++) {
            List<Integer> counts = Concurrently.collect(16, t -> {
                int allowed = 0;
                for (int i = 0; i < 10; i++) {
                    if (rl.allow("a")) {
                        allowed++;
                    }
                }
                return allowed;
            });
            assertThat(counts.stream().mapToInt(Integer::intValue).sum()).as("phase %d", phase).isEqualTo(20);
            clock.advance(1000);
        }
    }

    @Test
    void shouldStayCorrectAfterManyOneOffKeysAndPassingTime() {
        RateLimiterApi rl = limiter(2, 1000);
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 200_000; i++) {
                String key = "k" + i;
                assertThat(rl.allow(key)).isTrue();
                assertThat(rl.allow(key)).isTrue();
                assertThat(rl.allow(key)).isFalse();
                clock.advance(10);
            }
            // key i was used at START + 10 * i; now is START + 2_000_000
            assertThat(rl.allow("k0")).isTrue();
            assertThat(rl.allow("k0")).isTrue();
            assertThat(rl.allow("k0")).isFalse();
            assertThat(rl.allow("k199900")).isTrue();   // used exactly one window ago: expired
            assertThat(rl.allow("k199901")).isFalse();  // used 990 ms ago: still counts
            assertThat(rl.allow("k199999")).isFalse();
        });
    }

    @Test
    void shouldDecideQuicklyEvenWithAVeryLargeLimit() {
        RateLimiterApi rl = limiter(200_000, 10_000_000);
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 200_000; i++) {
                assertThat(rl.allow("hot")).isTrue();
                if (i % 10 == 0) {
                    clock.advance(1);
                }
            }
            for (int i = 0; i < 200_000; i++) {
                assertThat(rl.allow("hot")).isFalse();
            }
        });
    }
}
