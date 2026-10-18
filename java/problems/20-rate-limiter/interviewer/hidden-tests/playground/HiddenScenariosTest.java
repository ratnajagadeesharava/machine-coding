package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.RateLimiterApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {

    private RateLimiterApi limiter(int limit, long windowMillis, FakeClock clock) {
        return Candidate.of(() -> Solution.create(limit, windowMillis, clock));
    }

    @Test
    void shouldWorkWhenTheClockStartsAtZero() {
        FakeClock clock = new FakeClock(0);
        RateLimiterApi rl = limiter(1, 1000, clock);
        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isFalse();
        clock.advance(999);
        assertThat(rl.allow("a")).isFalse();
        clock.advance(1);
        assertThat(rl.allow("a")).isTrue();
    }

    @Test
    void shouldSupportAOneMillisecondWindow() {
        FakeClock clock = new FakeClock(5);
        RateLimiterApi rl = limiter(1, 1, clock);
        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isFalse();
        clock.advance(1);
        assertThat(rl.allow("a")).isTrue();
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void shouldExpireStaggeredRequestsOneByOne() {
        FakeClock clock = new FakeClock(0);
        RateLimiterApi rl = limiter(3, 1000, clock);
        assertThat(rl.allow("a")).isTrue();   // t = 0
        clock.advance(100);
        assertThat(rl.allow("a")).isTrue();   // t = 100
        clock.advance(100);
        assertThat(rl.allow("a")).isTrue();   // t = 200
        assertThat(rl.allow("a")).isFalse();
        clock.advance(800);
        assertThat(rl.allow("a")).isTrue();   // t = 1000: t = 0 left
        assertThat(rl.allow("a")).isFalse();
        clock.advance(100);
        assertThat(rl.allow("a")).isTrue();   // t = 1100: t = 100 left
        assertThat(rl.allow("a")).isFalse();
        clock.advance(100);
        assertThat(rl.allow("a")).isTrue();   // t = 1200: t = 200 left
        assertThat(rl.allow("a")).isFalse();
    }

    @Test
    void persistentRetriesShouldNotDelayTheNextAllowedRequest() {
        FakeClock clock = new FakeClock(10_000);
        RateLimiterApi rl = limiter(1, 1000, clock);
        assertThat(rl.allow("a")).isTrue();
        for (int i = 0; i < 9; i++) {
            clock.advance(100);
            assertThat(rl.allow("a")).isFalse();
        }
        clock.advance(100);
        assertThat(rl.allow("a")).isTrue();
    }

    @Test
    void oneKeysExpiryShouldNotAffectAnotherKey() {
        FakeClock clock = new FakeClock(0);
        RateLimiterApi rl = limiter(1, 1000, clock);
        assertThat(rl.allow("A")).isTrue();   // t = 0
        clock.advance(500);
        assertThat(rl.allow("B")).isTrue();   // t = 500
        clock.advance(500);
        assertThat(rl.allow("A")).isTrue();   // t = 1000: A's request expired
        assertThat(rl.allow("B")).isFalse();  // B's has not
        clock.advance(500);
        assertThat(rl.allow("B")).isTrue();   // t = 1500
    }
}
